use ::core::fmt::*;
use mantle::kio;

struct DebugOutput;

impl Write for DebugOutput {
    fn write_str(&mut self, s: &str) -> Result {
        for c in s.bytes() {
            kio::debug_put_char(c);
        }
        Ok(())
    }
}

static mut DEBUG_OUT: DebugOutput = DebugOutput {};

pub fn out() -> &'static mut Write {
    unsafe {
        &mut DEBUG_OUT
    }
}

macro_rules! debug {
    ($fmt:expr) => (write!(::mantle::debug::out(), concat!("[debug] ", $fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (write!(::mantle::debug::out(), concat!("[debug] ", $fmt, "\n"), $($arg)*));
}

macro_rules! debugc {
    ($fmt:expr) => (write!(::mantle::debug::out(), concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (write!(::mantle::debug::out(), concat!($fmt, "\n"), $($arg)*));
}

macro_rules! debugnl {
    ($fmt:expr) => (write!(::mantle::debug::out(), concat!("[debug] ", $fmt)));
    ($fmt:expr, $($arg:tt)*) => (write!(::mantle::debug::out(), concat!("[debug] ", $fmt), $($arg)*));
}
