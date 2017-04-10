// a mantle surrounds the core

extern crate rlibc;

#[macro_use]
pub mod debug;
pub mod kernel;
pub mod kio;
pub mod start;
pub mod calls;
pub mod concurrency;

pub use self::kernel::KError;
pub use self::calls::*;

pub fn debug() -> &'static mut ::core::fmt::Write {
    debug::out()
}
