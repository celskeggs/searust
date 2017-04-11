#![feature(lang_items)]
#![feature(naked_functions)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(drop_types_in_const)]
#![no_std]

#[macro_use]
pub mod mantle;
mod kobject;
mod crust;
mod memory;
mod drivers;

use core::fmt::Write;

pub fn main(bootinfo: &mantle::kernel::BootInfo) {
    match drivers::vga::VGAOutput::default() {
        Ok(mut screen) => {
            writeln!(screen, "Hello, world!").unwrap();
            crust::start::print_bootinfo(&mut screen, bootinfo).unwrap();
            memory::untyped::get_allocator().print_info(&mut screen).unwrap();
        }
        Err(err) => panic!("could not set up default VGA output: {:?}", err)
    }
}
