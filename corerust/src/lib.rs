#![feature(lang_items)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(drop_types_in_const)]
#![no_std]

#[macro_use]
mod debug;
pub mod sel4;
mod objs;
mod kobj;
mod device;
mod memory;
mod boot;
mod caps;
mod concurrency;

const VGA_BUFFER: usize = 0xb8000;

pub fn main() {
	match device::get_device_page(VGA_BUFFER) {
		Ok(page) => {
			writeln!(sel4::out(), "device page: obtained {:?}!", page);
		}, Err(err) => {
			panic!("Error: {:?}", err);
		}
	}
}
