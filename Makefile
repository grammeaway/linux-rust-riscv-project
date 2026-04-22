# Top-level driver for the RISC-V kernel module harness.
#
# Typical flow:
#   make           # build modules, stage them into rootfs, repack initramfs
#   make run       # boot under QEMU
#   make clean     # clean each module's build output
#
# Override on the command line, e.g. `make KDIR=/path/to/linux run`.

KDIR      ?= $(HOME)/code/linux
KERNEL    ?= $(KDIR)/arch/riscv/boot/Image
ROOTFS    ?= rootfs
INITRAMFS ?= initramfs.cpio.gz
QEMU      ?= qemu-system-riscv64

# Any top-level directory containing a Kbuild file is treated as a module.
MODULE_DIRS := $(patsubst %/Kbuild,%,$(wildcard */Kbuild))

QEMU_ARGS ?= -machine virt \
             -nographic \
             -m 512M \
             -smp 2 \
             -kernel $(KERNEL) \
             -initrd $(INITRAMFS) \
             -append "console=ttyS0 earlycon"

.PHONY: all help modules install-modules initramfs run clean $(MODULE_DIRS)

all: initramfs

help:
	@echo "Targets:"
	@echo "  all (default)    build modules, stage into $(ROOTFS)/lib/modules, repack $(INITRAMFS)"
	@echo "  modules          build every out-of-tree module: $(MODULE_DIRS)"
	@echo "  install-modules  copy freshly-built .ko files into $(ROOTFS)/lib/modules/"
	@echo "  initramfs        install-modules, then repack $(INITRAMFS)"
	@echo "  run              boot the guest under QEMU (exit with Ctrl-a x)"
	@echo "  clean            run 'make clean' in each module directory"
	@echo ""
	@echo "Overridable variables:"
	@echo "  KDIR=$(KDIR)"
	@echo "  KERNEL=$(KERNEL)"
	@echo "  INITRAMFS=$(INITRAMFS)"

modules: $(MODULE_DIRS)

$(MODULE_DIRS):
	$(MAKE) -C $@ KDIR=$(KDIR)

install-modules: modules
	@mkdir -p $(ROOTFS)/lib/modules
	@set -e; for d in $(MODULE_DIRS); do \
		for ko in $$d/*.ko; do \
			[ -e "$$ko" ] || continue; \
			echo "  INSTALL $$ko -> $(ROOTFS)/lib/modules/"; \
			cp $$ko $(ROOTFS)/lib/modules/; \
		done; \
	done

initramfs: install-modules
	@echo "  PACK    $(INITRAMFS)"
	@cd $(ROOTFS) && find . -print0 | cpio --null -o --format=newc --quiet | gzip -9 > ../$(INITRAMFS)

run:
	@test -f $(KERNEL)    || { echo "ERROR: kernel image not found at $(KERNEL) (set KDIR=... or cross-compile first)"; exit 1; }
	@test -f $(INITRAMFS) || { echo "ERROR: $(INITRAMFS) missing; run 'make' first"; exit 1; }
	$(QEMU) $(QEMU_ARGS)

clean:
	@for d in $(MODULE_DIRS); do \
		$(MAKE) -C $$d clean KDIR=$(KDIR); \
	done
