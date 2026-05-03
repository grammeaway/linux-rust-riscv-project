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
    str::Formatter
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
            let time = read_csr!(time);
            let cycle = read_csr!(cycle);
            let instret = read_csr!(instret);

            let mut stack_buf = [0u8; 256];
            let mut formatter = Formatter::new(&mut stack_buf);

            write!(
                &mut formatter,
                "RISC-V CSRs - time: {}, cycle: {}, instret: {}\n",
                time, cycle, instret
            )
            .map_err(|_| EINVAL)?;

            let written = formatter.bytes_written();
            buffer.extend_from_slice(&stack_buf[..written], GFP_KERNEL)?;

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

}

#[pinned_drop]
impl PinnedDrop for RustMiscDevice {
    fn drop(self: Pin<&mut Self>) {
        dev_info!(self.dev, "Exiting the Rust Misc Device Sample\n");
    }
}

impl RustMiscDevice {
}
