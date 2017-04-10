#![feature(lang_items)]
#![feature(naked_functions)]
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
mod vga;

use core::fmt::Write;

pub fn main() {
    match vga::VGAOutput::default() {
        Ok(mut screen) => {
            writeln!(screen, "Hello, world!");
        }
        Err(err) => panic!("could not set up default VGA output: {:?}", err)
    }
}
