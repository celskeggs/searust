#!/bin/bash -e

echo "Use Ctrl-A x to quit qemu"
qemu-system-x86_64 -m 256 -nographic -kernel sysroot/boot/sel4-dev -initrd sysroot/boot/init.elf
