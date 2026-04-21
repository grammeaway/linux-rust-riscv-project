# RISC-V cross-compiled Linux kernel project 

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
