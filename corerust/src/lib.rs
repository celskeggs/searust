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
    let mut com1: drivers::serial::HardwareSerial = drivers::serial::COM1.configure(115200);
    com1.send_str("HELLO SERIAL WORLD\n");
    let line = com1.recv_line();
    com1.send_str("RECEIVED: '");
    com1.send_str(line.as_str());
    com1.send_str("'\n");
}
