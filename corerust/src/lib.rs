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
mod vspace;
mod concurrency;

const VGA_BUFFER: usize = 0xb8000;

pub fn main() {
	// VGA test
	let page = device::get_device_page(VGA_BUFFER).unwrap();
	writeln!(sel4::out(), "device page: obtained {:?}!", page);
	let page2 = match page.map_into_vspace(true) {
		Ok(mut mapping) => {
			{
				let array = mapping.get_array();
				for i in 0..200 {
					array[i] = 0x55;
				}
			}
			mapping.unmap()
		}, Err((page, err)) => {
			page
		}
	};
	device::return_device_page(VGA_BUFFER, page2);
}
