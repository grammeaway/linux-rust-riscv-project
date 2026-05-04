// SPDX-License-Identifier: GPL-2.0
// Copyright (C) 2024 Google LLC.

use kernel::{
    device::Device,
    fs::{File, Kiocb},
    iov::IovIterDest,
    miscdevice::{MiscDevice, MiscDeviceOptions, MiscDeviceRegistration},
    new_mutex,
    prelude::*,
    sync::{aref::ARef, Mutex},
    str::Formatter,
    transmute::AsBytes,
    uaccess::UserSlice, 
    ioctl::_IOR,
};
use core::arch::asm; // For the `asm!` macro.
use core::fmt::Write;

module! {
    type: RustMiscDeviceModule,
    name: "rust_misc_device",
    authors: ["Victor Gram Thomsen"],
    description: "Rust misc device sample",
    license: "GPL",
}

#[pin_data]
struct RustMiscDeviceModule {
    #[pin]
    _miscdev: MiscDeviceRegistration<RustMiscDevice>,
}


#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct RvcpuSnapshot {
    time: u64,
    cycle: u64,
    instret: u64,
}

unsafe impl AsBytes for RvcpuSnapshot {}

const RVCPU_IOC_SNAPSHOT: u32 = _IOR::<RvcpuSnapshot>('|' as u32, 0x80);

macro_rules! read_csr { 
    ($csr:ident) => {{ 
        let value: u64;
        // SAFETY: reading a CSR is a pure read with no side effects
        unsafe { 
            asm!(
                concat!("csrr {0}, ", stringify!($csr)),
                out(reg) value 
            );
        }
        value
    }};
}

#[cfg(target_arch = "riscv64")]
fn take_snapshot() -> RvcpuSnapshot {
    RvcpuSnapshot {
        time: read_csr!(time),
        cycle: read_csr!(cycle),
        instret: read_csr!(instret),
    }
}


impl kernel::InPlaceModule for RustMiscDeviceModule {
    fn init(_module: &'static ThisModule) -> impl PinInit<Self, Error> {
        pr_info!("Initialising Rust Misc Device Sample\n");

        let options = MiscDeviceOptions {
            name: c"rvcpu",
        };

        try_pin_init!(Self {
            _miscdev <- MiscDeviceRegistration::register(options),
        })
    }
}

struct Inner {
    buffer: KVVec<u8>,
}

#[pin_data(PinnedDrop)]
struct RustMiscDevice {
    #[pin]
    inner: Mutex<Inner>,
    dev: ARef<Device>,
}

#[vtable]
impl MiscDevice for RustMiscDevice {
    type Ptr = Pin<KBox<Self>>;

    fn open(_file: &File, misc: &MiscDeviceRegistration<Self>) -> Result<Pin<KBox<Self>>> {

        let dev = ARef::from(misc.device());
        dev_info!(dev, "Opening Rust Misc Device Sample\n");


        let mut buffer = KVVec::new();

        #[cfg(target_arch = "riscv64")]
        {
            let snap = take_snapshot();
            buffer.extend_from_slice(snap.as_bytes(), GFP_KERNEL)?;
        }

        KBox::try_pin_init(
            try_pin_init! {
                RustMiscDevice {
                    inner <- new_mutex!(Inner {
                        buffer: buffer,
                    }),
                    dev: dev,
                }
            },
            GFP_KERNEL,
        )
    }

    fn read_iter(mut kiocb: Kiocb<'_, Self::Ptr>, iov: &mut IovIterDest<'_>) -> Result<usize> {
        let me = kiocb.file();
        dev_info!(me.dev, "Reading from Rust Misc Device Sample\n");

        let inner = me.inner.lock();
        // Read the buffer contents, taking the file position into account.
        let read = iov.simple_read_from_buffer(kiocb.ki_pos_mut(), &inner.buffer)?;

        Ok(read)
    }

fn ioctl(me: Pin<&RustMiscDevice>, _file: &File, cmd: u32, arg: usize) -> Result<isize> {
    dev_info!(me.dev, "IOCTL on Rust Misc Device Sample (cmd: {})\n", cmd);

    match cmd {
        RVCPU_IOC_SNAPSHOT => {
            #[cfg(target_arch = "riscv64")]
            {
                let snap = take_snapshot();
                let user_arg = UserPtr::from_addr(arg);
                let size = core::mem::size_of::<RvcpuSnapshot>();
                UserSlice::new(user_arg, size)
                    .writer()
                    .write::<RvcpuSnapshot>(&snap)?;
            }
            Ok(0)
        }
        _ => {
            dev_err!(me.dev, "Unrecognised IOCTL command: {}\n", cmd);
            Err(ENOTTY)
        }
    }
}

}

#[pinned_drop]
impl PinnedDrop for RustMiscDevice {
    fn drop(self: Pin<&mut Self>) {
        dev_info!(self.dev, "Exiting the Rust Misc Device Sample\n");
    }
}

impl RustMiscDevice {
}
