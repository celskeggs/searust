#![feature(lang_items)]
#![feature(asm)]
#![no_std]

pub mod sel4;

fn sel4_debug_put_str(s : &str) {
	for c in s.bytes() {
		sel4::sel4_debug_put_char(c);
	}
}

pub fn main() {
	sel4_debug_put_str("Hello, World!\n");
}
