all: init.elf

target/x86_64-unknown-linux-gnu/debug/libsearust_core.a: src/lib.rs src/build.rs src/sel4.rs src/device.rs src/memory.rs src/boot.rs src/caps.rs src/kobj.rs src/objs.rs src/concurrency.rs src/debug.rs src/vspace.rs src/vga.rs
	cargo build --target=x86_64-unknown-linux-gnu

init.elf: target/x86_64-unknown-linux-gnu/debug/libsearust_core.a
	$(LD) --gc-sections -o $@ $^

.PHONY: clean install

clean:
	rm -f init.elf
	cargo clean

install: init.elf
	install -D -m 644 init.elf $(SYSROOT)/boot/init.elf
