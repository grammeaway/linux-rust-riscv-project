# RISC-V + Rust cross-compiled Linux kernel project 

## WIP (and aggresively so) - not ready for use yet
For now, this is mainly a scratch space for me to experiment with cross-compiling the Linux kernel and busybox for RISC-V, and then running it in QEMU. As I make progress, I'll update the repo and README with more instructions and details.

Progress so far:
- Successfully cross-compiled a Linux kernel for RISC-V and generated an initramfs with busybox.
- Able to boot the kernel in QEMU and get a console.
- Ran some basic busybox commands like `ls`, `echo`, and `cat` in the QEMU console.

Coming next:
- Add more utilities to the initramfs and test them.
- Write some kernel modules in Rust and load them in the kernel.


## Assorted setup instructions and notes for myself (and anyone else who might find this useful):
Setup requirements:

- RISC-V compiled busybox binary in `rootfs/bin/busybox`
- RISC-V compiled Linux kernel in `linux/arch/riscv/boot/Image` (or whatever path you have built it to)
- `initramfs.cpio.gz` generated from the `rootfs` directory


QEMU run command:
```bash
qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -m 512M \
    -smp 2 \
    -kernel linux/arch/riscv/boot/Image \
    -initrd initramfs.cpio.gz \
    -append "console=ttyS0 earlycon"
```


rootfs build command:
```bash
cd rootfs
find . -print0 | cpio --null -o --format=newc | gzip -9 > ../initramfs.cpio.gz
```
