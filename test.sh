#!/bin/bash -e

echo "Use Ctrl-A x to quit qemu"
# -nographic 
qemu-system-x86_64 -m 4096 -display sdl -serial stdio -kernel sysroot/boot/sel4-dev -initrd sysroot/boot/init.elf
