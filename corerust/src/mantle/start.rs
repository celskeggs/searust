use mantle::kernel;
use mantle::kio;
use ::core;

static mut DEFAULT_STACK: [u8; 65536] = [0; 65536];

extern {
    fn mantle_main(bi: &kernel::BootInfo, executable_start: usize);
}

#[no_mangle]
#[naked]
pub unsafe extern fn _start() {
    asm!("
.extern __executable_start
    add $$65536, %rsp
    movq $$__executable_start, %rsi
    call mantle_start
_halt:
    jmp _halt
" ::"{rsp}" (&DEFAULT_STACK):: "volatile");
}

#[no_mangle]
pub unsafe extern fn mantle_start(bootinfo_addr: usize, executable_start: usize) {
    let bootinfo = unsafe {
        (bootinfo_addr as *const kernel::BootInfo).as_ref().unwrap()
    };
    mantle_main(bootinfo, executable_start);
    panic!("returned from main!");
}

#[lang = "eh_personality"]
extern fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    debug!("panicked at {}:{}: {}", file, line, fmt);
    for c in "[panic] HANG\n".bytes() {
        kio::debug_put_char(c);
    }
    loop {}
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    panic!("cannot unwind");
}
