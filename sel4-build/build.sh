#!/bin/bash -e

if [ ! -e kernel ]
then
	git clone https://github.com/celskeggs/seL4.git kernel
else
	(cd kernel && git fetch origin)
fi
(cd kernel && git checkout e26007550636842893605074b9fa2ba896bd93b3 && rm -rf -- * && git checkout .)

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

SEL4_ARCH=x86_64 ARCH=x86 PLAT=pc99 make V=2 clean
