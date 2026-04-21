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

    - run `ARCH=riscv CROSS_COMPILE=riscv64-linux-gnu- make` to build the kernel

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


Created: 2026-04-19 18:29:48
