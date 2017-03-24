#![feature(lang_items)]
#![feature(asm)]
#![no_std]

pub mod sel4;

use core::fmt::Write;

struct DebugOutput {
}

impl core::fmt::Write for DebugOutput {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		for c in s.bytes() {
			sel4::sel4_debug_put_char(c);
		}
		Ok(())
	}
}

pub fn main() {
	let mut writer = DebugOutput {};
	writeln!(writer, "ABC");
	writeln!(writer, "a number: {} and {}", 42, 1.0 / 3.0);
}
