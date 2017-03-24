all: build/init.elf

build/_start.s: src/_start.S
	mkdir -p build
	$(CPP) -nostdinc -D HAVE_SEL4_CONFIG -I $(SYSROOT)/usr/include/ src/_start.S >build/_start.s

build/_start.o: build/_start.s
	$(AS) --64 build/_start.s -o build/_start.o

target/x86_64-unknown-linux-gnu/debug/libsearust_core.a: src/lib.rs src/build.rs
	cargo build --target=x86_64-unknown-linux-gnu

build/init.elf: build/_start.o target/x86_64-unknown-linux-gnu/debug/libsearust_core.a
	$(LD) --gc-sections -o $@ $^

.PHONY: clean install

clean:
	rm -f build/init.elf build/_start.o build/_start.s
	cargo clean

install: build/init.elf
	install -D -m 644 build/init.elf $(SYSROOT)/boot/init.elf
