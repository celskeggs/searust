#!/bin/bash -e
rm -f init.elf
cargo build --target=x86_64-unknown-linux-gnu
ld --gc-sections target/x86_64-unknown-linux-gnu/debug/libsearust_core.a -o init.elf
install -D -m 644 init.elf $SYSROOT/boot/init.elf
