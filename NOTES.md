# RISC-V hobby project, log 

Working from: Arch Linux, x86_64

## Step 1: Cross-compilations tools 

- riscv64-linux-gnu-gcc - installed through pacman (extra)

- llvm - pacman (extra)

- qemu-base and qemu-system-riscv - pacman (extra)

## Step 2: A Linux kernel

- Download from GitHub, limit fetch depth to 1, go for whichever tag is the latest stable - runnning v7.0 in my case 

- Configure it to cross-compile for RISC-V, in menuconfig:

    - run `make ARCH=riscv CROSS_COMPILE=riscv64-linux-gnu- menuconfig` in the kernel source directory

    - Save and exit immediately, since the default config is fine for our purposes

    - run `ARCH=riscv CROSS_COMPILE=riscv64-linux-gnu- make -j$(nproc)` to build the kernel (the -j flag is optional but speeds up the build by using multiple cores)

        - had to install bc for this to work - you might have other missing deps, install as needed

    - Pretty giddy about building a kernel for the first time.

## Step 3: busybox

- Download BusyBox source code - busybox.net, running v1.37.0

    - Additional deps: ncurses-devel (on Arch as just ncurses)

        - subseuently, applied following patch to script: https://stackoverflow.com/questions/78491346/busybox-build-fails-with-ncurses-header-not-found-in-archlinux-spoiler-i-alrea 

        - Just involved fixing a shell script, giving it's main function an int return type

- Configure it to Cross-compile for RISC-V, in menuconfig:

    - run `make menuconfig` in the busybox source directory

    - Toggle Build static binary (no shared libs)

    - Set Cross Compiler prefix to riscv64-linux-gnu-

    - Save and exit

    - run `make` to build the static binary



- In case of error: libbb/hash_md5_sha.c: In function 'sha1_end':
libbb/hash_md5_sha.c:1316:35: error: 'sha1_process_block64_shaNI' undeclared (first use in this function); did you mean 'sha1_process_block64'?
 1316 |          || ctx->process_block == sha1_process_block64_shaNI 


    - You'll need to go into the code of libbb/hash_md5_sha.c, and add additional guards around the sha1_process_block64_shaNI function, to only use it if the architecture supports it (which RISC-V doesn't)

    - OR: Disable the use of sha hwaccel in the busybox configuration. I went with the C fix, since writing some C for the first time in years made me a bit giddy.


- In case of a bunch of TC relaed errors (kernel header drift from 6.5), just disable it in the busybox configuration, since it's not really needed for a simple root FS




## Step 4: Create a root filesystem

- Create a directory to hold the root filesystem, and copy the busybox binary into it, along with any necessary libraries (use ldd to find out which ones you need)

- See project for structure 

- Remember to add symlinks for the busybox applets, so that you can use them from the command line

- Pack the root filesystem into an initramfs image using cpio, and gzip it to create an initramfs.gz file


## Step 5: Booting with QEMU

- See README


## Step 6: Rust for Linux toolchain
- Use the Linux kernel scripts and tools, to check your system and environment for whether it's ready for building Rust code for the kernel, and if not, what you need to do to get it ready.

- In my case, installed rust-source and rust-bindgen from pacman, and then added the rust-src component to my Rust toolchain using rustup

## Step 7: configmenu confs for Rust for Linux

- Enable Rust Support in the general setup section, in the menuconfig for the kernel

- In the Kernel hacking section, enable sample kernel code, and in the subsequent menu for that, enable the Rust sample code. You'll have multiple options for the Rust sample code. For a start, I just went with "Minimal" (a hello world module), configured as a built-in module, since it's the simplest to get up and running.

- Save and exit, and then rebuild using: `ARCH=riscv CROSS_COMPILE=riscv64-linux-gnu- LLVM=1 make -j$(nproc)` - the LLVM=1 flag is needed to enable the use of the Rust toolchain for building the Rust code in the kernel, since it relies on LLVM for codegen. Without the LLVM flag, I got prompted for a lot of additional configuration options related to the Rust code in the kernel


## Step 8: Verify Rust code is running in the kernel

- Boot into the kernel in QEMU, and check the kernel logs: `dmesg | grep -i rust` - you should see something like:
```bash
[    0.758339] rust_minimal: Rust minimal sample (init)
[    0.758858] rust_minimal: Am I built-in? true
[    0.759365] rust_minimal: test_parameter: 1
``` 


## Step 9: A simple Rust kernel module

- Switch the Rust sample code in the menuconfig from the minimal built-in module, to a loadable module, and then rebuild the kernel with the same command as before

    - Double-check that "Enable Loadable Module Support" is enabled in the menuconfig, since it's required for loadable modules to work - should be a default option, can be seen on the very first page of the menuconfig

- After building, you should see a new .ko file in the same directory as the minimal module's .ko file - this is your Rust kernel module, which you can load using insmod and unload using rmmod, just like any other kernel module. You can also check the kernel logs again to see the output from your Rust module when it's loaded and unloaded.


- Copy the .ko file into your rootfs (at /lib/modules/), and re-make the initramfs image, so that it's included in the initramfs when you boot the kernel in QEMU. 


- Boot into the kernel in QEMU, and run: `insmod /lib/modules/rust_minimal.ko`. This should produce the output:

```bash 
~ # insmod /lib/modules/rust_minimal.ko
[    9.609871] rust_minimal: Rust minimal sample (init)
[    9.611867] rust_minimal: Am I built-in? false
[    9.613910] rust_minimal: test_parameter: 1 

```

    - You can pass parameters to the module when you load it with insmod, which can be useful for testing that parameter parsing is working correctly in your Rust code. For example, you could run `insmod /lib/modules/rust_minimal.ko test_parameter=42` to set the test_parameter to 42 instead of the default value of 1, and then check the kernel logs again to see that the new value is being used in the output from your module.

- Following the succesful load with `insmod`, you can then run `rmmod rust_minimal` to unload the module, which should produce the output:

```bash
~ # rmmod rust_minimal
[   89.664436] rust_minimal: My numbers are [72, 108, 200]
[   89.667130] rust_minimal: Rust minimal sample (exit)
```

- And just like that, you have a Rust kernel development environment and loop running.


## Step 10: Tinkering with the Rust code 

- Open the rust_minimal.rs file in the kernel source tree, and make some changes to the code 

- I added a new pr_info macro, a new test parameter, a pr_info printing said parameter, modified the Vector of numbers that gets printed when the module is unloaded

- For something with a bit more meat on it, I added a pr_info outputting the number of CPU IDs, the current CPU ID, a small time delta measurement, and the current jiffies value

    - All of these involved poking around in the Rust kernel modules, and playing around with their APIs and data structures

- After making changes, rebuild the kernel again with the same command as before, copy the new .ko file into your rootfs, remake the initramfs, and then boot into the kernel in QEMU to see your changes in action when you load and unload the module with insmod and rmmod. 


- My output after the changes looked like this:

```bash
~ # insmod /lib/modules/rust_minimal.ko 
[   32.465710] rust_minimal: Rust minimal sample (init)
[   32.467403] rust_minimal: Am I built-in? false
[   32.469608] rust_minimal: A new pr_info! macro with 42 and hello
[   32.472496] rust_minimal: test_parameter: 1
[   32.474030] rust_minimal: test_parameter_two: 2
[   32.475053] rust_minimal: measured delta: 25000 ns
[   32.476094] rust_minimal: nr_cpu_ids: 2
[   32.477645] rust_minimal: Current CPU ID: 0
[   32.478467] rust_minimal: jiffies = 4294900408
```

```bash
~ # rmmod rust_minimal
[  190.817228] rust_minimal: My numbers are [42, 120, 256]
[  190.819226] rust_minimal: Rust minimal sample (exit)
```

- Noting that the drop output matched the new Vector of numbers


## Step 11: An out-of-tree module 

- Made a copy of the rust minimal module in this repoitory (in the `my-module/` directory), and then modified it to create a new module name - left everything else the same for now.

- Made a Kbuild file for it, which is needed to build it as an out-of-tree module

- Made a Makefile for it, which uses the Kbuild system to build the module against the kernel source tree

- After a succesful build (had to rustup override to a stable Rust version, since I run the nightly toolchain by default), the flow was the same as before - copy the new .ko file into the rootfs, remake the initramfs, boot into the kernel in QEMU, and then load and unload the module with insmod and rmmod to see the output in the kernel logs.
