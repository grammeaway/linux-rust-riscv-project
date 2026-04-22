KDIR ?= ~/code/linux
PWD := $(shell pwd)

default:
	$(MAKE) -C $(KDIR) M=$(PWD) ARCH=riscv LLVM=1 modules

clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean
