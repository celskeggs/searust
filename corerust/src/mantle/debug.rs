use ::core;
use ::core::fmt::*;
use mantle::concurrency::SingleThreaded;
use mantle::kio;
use core::ops::DerefMut;
use memory::Box;

struct DebugOutput;

const DEBUG_ON: bool = false;

impl Write for DebugOutput {
    fn write_str(&mut self, s: &str) -> Result {
        if DEBUG_ON {
            for c in s.bytes() {
                kio::debug_put_char(c);
            }
            let mut rmut: core::cell::RefMut<Option<&'static mut Write>> = DEBUG_MIRROR.get().borrow_mut();
            let rmutr: &mut Option<&'static mut Write> = rmut.deref_mut();
            if let &mut Some(ref mut w) = rmutr {
                w.write_str(s);
            }
        }
        Ok(())
    }
}

static mut DEBUG_OUT: DebugOutput = DebugOutput {};
static DEBUG_MIRROR: SingleThreaded<core::cell::RefCell<Option<&'static mut Write>>> = SingleThreaded(core::cell::RefCell::new(None));

pub fn out() -> &'static mut Write {
    unsafe {
        &mut DEBUG_OUT
    }
}

pub fn set_mirror<T: Write + 'static>(w: T) {
    // TODO: clean this up
    let rf: *mut T = Box::into_raw(Box::new(w));
    let bxr: &mut T = unsafe { rf.as_mut() }.unwrap();
    *DEBUG_MIRROR.get().borrow_mut() = Some(bxr);
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
