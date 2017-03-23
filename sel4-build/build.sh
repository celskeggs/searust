#!/bin/bash -e

if [ "$SYSROOT" = '' ]
then
	echo "Sysroot not specified!" 1>&2
	exit 1
fi

if [ ! -e kernel ]
then
	git clone https://github.com/celskeggs/seL4.git kernel
else
	(cd kernel && git fetch origin)
fi
(cd kernel && git checkout ef99634247b859ae8aafb1e12f052780363a7dbd && rm -rf -- * && git checkout .)

if [ ! -e tools/common ]
then
	mkdir -p tools
	(cd tools && git clone https://github.com/celskeggs/common-tool.git common)
else
	(cd tools/common && git fetch origin)
fi
(cd tools/common && git checkout cb4a1508e44338552ac7f51a2f27fc92e28786b7)

if [ ! -e tools/kbuild ]
then
	(cd tools && git clone https://github.com/seL4/kbuild-tool kbuild)
else
	(cd tools/kbuild && git fetch origin)
fi
(cd tools/kbuild && git checkout 820f7efb4fbceeb1d0223f48f34dacfe8378cfdb)

rm -rf build images stage include
SEL4_ARCH=x86_64 ARCH=x86 PLAT=pc99 make -j4
install -D -m 644 stage/x86/pc99/kernel.elf $SYSROOT/boot/sel4-dev
