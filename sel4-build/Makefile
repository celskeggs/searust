#
# Copyright 2014, NICTA
#
# This software may be distributed and modified according to the terms of
# the BSD 2-Clause license. Note that NO WARRANTY is provided.
# See "LICENSE_BSD2.txt" for details.
#
# @TAG(NICTA_BSD)
#

# special commands:
#   make menuconfig
#   make silentoldconfig
#   make simulate-ia32

all: common kernel_elf

PHONY += libsel4

include .config
include tools/common/project.mk

LIBSEL4_ROOT_PATH=$(KERNEL_ROOT_PATH)/libsel4

# includes installation into sysroot
libsel4: setup
	@echo "[LIBSEL4]"
	$(Q)$(MAKE) $(MAKE_SILENT) -C $(LIBSEL4_ROOT_PATH) -f $(LIBSEL4_ROOT_PATH)/Makefile \
		SOURCE_DIR=$(LIBSEL4_ROOT_PATH) O=$(KBUILD_ROOT) V=$(V) \
		TOOLPREFIX=$(CONFIG_CROSS_COMPILER_PREFIX:"%"=%) \
		HAVE_AUTOCONF=1 NO_PRESERVE_TIMESTAMPS=1 CFLAGS="$(CFLAGS)" \
		AUTOCONF_DIR=${PWD}/include/generated
	@echo "[LIBSEL4] done."

images/kernel.elf: kernel_elf
	objcopy -O elf32-i386 $< $@

install: common images/kernel.elf libsel4
	install -D -m 644 images/kernel.elf $(SYSROOT)/boot/sel4-dev
