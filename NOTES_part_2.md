# RISC-V hobby project, log, part 2 

Working from the basis of NOTES_part_1.md, which contains the initial steps of cross-compiling a Linux kernel and busybox for RISC-V, and running it in QEMU.

## Step 1: Sanity check of old setup
Just re-running the old setup instructions to make sure everything still works, and to refresh my memory on the steps. Successfully booted into QEMU and got a console, so we're good to go.

## Step 2: Preliminary research for a misc device driver in Rust

- Have a look at `linux/samples/rust/rust_misc_devices.rs` for a simple misc device driver example in Rust. This will be our starting point for writing a kernel module in Rust.


## Step 3: Copying and stripping down the Rust misc device driver example

- Copy the `rust_misc_devices.rs` file to a new module in the project dir. 

- Strip down the example by removing write_iter/ioctl/set_value/get_value/hello, and rename the device to `rvcpu`.

- Get it compiling and loading as a useless /dev/rvcpu that returns an empty buffer. 

- When built and loaded, `insmod` should show the module is loaded

- While in its loaded stage, run `ls /sys/class/misc/`, and you should see `rvcpu` listed as a misc device.

## Step 4: Why is there no /dev/rvcpu?

- The reason there is no /dev/rvcpu is because the misc device driver example does not include the necessary code to create a device node in /dev. This would in a real driver be done using mdev, but for simplicity we can create the device node manually using `mknod` after loading the module.

### Step 4.1: Adding the mknod symbolic link to the module
Our original busybox config didn't include mknod, so we need to add it in order to create the device node for /dev/rvcpu.

- `cd` into your `rootfs/bin` directory and run `ln -s busybox mknod` to create a symbolic link for mknod.

- Re-build the rootfs and re-run QEMU. 


### Step 4.2: Creating the device node for /dev/rvcpu

- After booting into QEMU and loading the module, run:

```bash
cat /sys/class/misc/rvcpu/dev
```
This will output something like `10:258`, which is the major and minor number for the device.

- Now we can create the device node using `mknod`:

```bash
mknod /dev/rvcpu c 10 258
```

- Now we should have a /dev/rvcpu device node that we can interact with.

```bash 
cat /dev/rvcpu
```

## Step 5: Making step 4 obsolete
The `mknod` flow is good to go through for learning purposes, but it's not ideal for us to do on every QEMU launch. This is luckily pretty simple to address, by adding a new line to our `init` script.

Following on from part 1, that should currently look like this:

```bash
#!/bin/sh
mount -t proc proc /proc
mount -t sysfs sysfs /sys
exec /bin/sh
```

Add the line:
```bash
mount -t devtmpfs devtmpfs /dev
```
to the end of the `init` script, before the `exec /bin/sh` line. This will automatically mount the devtmpfs filesystem on /dev, which will create device nodes for all devices registered with the kernel, including our /dev/rvcpu misc device.

To validate that this works, we can re-build the rootfs and re-run QEMU. After booting into QEMU and loading the module, we should now see that /dev/rvcpu is automatically created without needing to run `mknod`.

## Step 6: Time to read some CSR registers, and write them to the device buffer

### Step 6.1: Adding the `read_csr` macro to our module
Steal the `read_csr` macro from the `my_csr_module` example from part 1, and add it to our `rvcpu` module. Remember to impost the `core::arch::asm` module to get access to the `asm!` macro.

Make sure to architecure-guard the code with `#[cfg(target_arch = "riscv64")]` to ensure it only compiles on RISC-V.

### Step 6.2: Writing the readings to our device buffer
We'll need to refactor a bit here, since the current `KVVec`buffer, doesn't support us writing to it directly via e.g. the `write!` macro. We'll be using a stack buffer to write the readings to, and then copy that buffer to the `KVVec`buffer before returning it to userspace.

