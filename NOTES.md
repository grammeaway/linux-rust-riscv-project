# RISC-V hobby project, log 

Working from: Arch Linux, x86_64

# Step 1: Cross-compilations tools 

- riscv64-linux-gnu-gcc - installed through pacman (extra)

- llvm - pacman (extra)

- qemu-base and qemu-system-riscv - pacman (extra)

# Step 2: A Linux kernel

- Download from GitHub, limit fetch depth to 1, go for whichever tag is the latest stable - runnning v7.0 in my case 

- Configure it to cross-compile for RISC-V, in menuconfig:

    - run `make ARCH=riscv CROSS_COMPILE=riscv64-linux-gnu- menuconfig` in the kernel source directory

    - Save and exit immediately, since the default config is fine for our purposes

    - run `ARCH=riscv CROSS_COMPILE=riscv64-linux-gnu- make -j$(nproc)` to build the kernel (the -j flag is optional but speeds up the build by using multiple cores)

        - had to install bc for this to work - you might have other missing deps, install as needed

    - Pretty giddy about building a kernel for the first time.

# Step 3: busybox

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




# Step 4: Create a root filesystem

- Create a directory to hold the root filesystem, and copy the busybox binary into it, along with any necessary libraries (use ldd to find out which ones you need)

- See project for structure 

- Remember to add symlinks for the busybox applets, so that you can use them from the command line

- Pack the root filesystem into an initramfs image using cpio, and gzip it to create an initramfs.gz file


# Step 5: Booting with QEMU

- See new reference repository


# Step 6: Rust for Linux toolchain
- Use the Linux kernel scripts and tools, to check your system and environment for whether it's ready for building Rust code for the kernel, and if not, what you need to do to get it ready.

- In my case, installed rust-source and rust-bindgen from pacman, and then added the rust-src component to my Rust toolchain using rustup

# Step 7: configmenu confs for Rust for Linux

- Enable Rust Support in the general setup section, in the menuconfig for the kernel

- In the Kernel hacking section, enable sample kernel code, and in the subsequent menu for that, enable the Rust sample code. You'll have multiple options for the Rust sample code. For a start, I just went with "Minimal" (a hello world module), configured as a built-in module, since it's the simplest to get up and running.

- Save and exit, and then rebuild using: `ARCH=riscv CROSS_COMPILE=riscv64-linux-gnu- LLVM=1 make -j$(nproc)` - the LLVM=1 flag is needed to enable the use of the Rust toolchain for building the Rust code in the kernel, since it relies on LLVM for codegen. Without the LLVM flag, I got prompted for a lot of additional configuration options related to the Rust code in the kernel


# Step 8: Verify Rust code is running in the kernel

- Boot into the kernel in QEMU, and check the kernel logs: `dmesg | grep -i rust` - you should see something like:
```bash
[    0.758339] rust_minimal: Rust minimal sample (init)
[    0.758858] rust_minimal: Am I built-in? true
[    0.759365] rust_minimal: test_parameter: 1
``` 
