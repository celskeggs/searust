macro_rules! debug {
    ($fmt:expr) => (write!(::sel4::out(), concat!("[debug] ", $fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (write!(::sel4::out(), concat!("[debug] ", $fmt, "\n"), $($arg)*));
}

macro_rules! debugc {
    ($fmt:expr) => (write!(::sel4::out(), concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (write!(::sel4::out(), concat!($fmt, "\n"), $($arg)*));
}

macro_rules! debugnl {
    ($fmt:expr) => (write!(::sel4::out(), concat!("[debug] ", $fmt)));
    ($fmt:expr, $($arg:tt)*) => (write!(::sel4::out(), concat!("[debug] ", $fmt), $($arg)*));
}
