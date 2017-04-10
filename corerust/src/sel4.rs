extern crate rlibc;

#[cfg(target_arch = "x86_64")]
mod coretypes {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]

    pub type c_char = i8;
    pub type c_uchar = u8;
    pub type c_short = i16;
    pub type c_ushort = u16;
    pub type c_int = i32;
    pub type c_uint = u32;
    pub type c_long = i64;
    pub type c_ulong = u64;
}

pub mod libsel4 {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]
    include!(concat!(env!("OUT_DIR"), "/libsel4.rs"));
}

impl ::core::fmt::Display for libsel4::seL4_SlotRegion {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "[{}, {})", self.start, self.end)
    }
}

// core kernel I/O functions

#[cfg(target_arch = "x86_64")]
unsafe fn x64_sys_send_recv(syscall: i64, dest: u64, info: u64, mr0: u64, mr1: u64,
                            mr2: u64, mr3: u64) -> (u64, u64, u64, u64, u64, u64) {
    let info_out;
    let dest_out;
    let mr0_out;
    let mr1_out;
    let mr2_out;
    let mr3_out;
    asm!(
		"movq %rsp, %rbx\nsyscall\nmovq %rbx, %rsp\n"
		: "={rsi}"(info_out),
		"={r10}"(mr0_out),
		"={r8}"(mr1_out),
		"={r9}"(mr2_out),
		"={r15}"(mr3_out),
		"={rdi}"(dest_out)
		: "{rdx}"(syscall),
		"{rdi}"(dest),
		"{rsi}"(info),
		"{r10}"(mr0),
		"{r8}"(mr1),
		"{r9}"(mr2),
		"{r15}"(mr3)
		: "rcx", "rbx", "r11", "memory"
		: "volatile"
	);
    (dest_out, info_out, mr0_out, mr1_out, mr2_out, mr3_out)
}

// IPC buffer accessors

// Layout: TODO synchronize this!
/*  pub struct seL4_IPCBuffer_ {
        pub tag: seL4_MessageInfo_t,                @   0 u64
        pub msg: [seL4_Word; 120usize],             @   1 u64 <---
        pub userData: seL4_Word,                    @ 121 u64
        pub caps_or_badges: [seL4_Word; 3usize],    @ 122 u64 <---
        pub receiveCNode: seL4_CPtr,                @ 125 u64
        pub receiveIndex: seL4_CPtr,                @ 126 u64
        pub receiveDepth: seL4_Word,                @ 127 u64
    }   */

const CAPS_OR_BADGES_LEN: usize = 3;

#[cfg(target_arch = "x86_64")]
pub fn sel4_set_cap(i: u32, cptr: usize) {
    assert!((i as usize) < CAPS_OR_BADGES_LEN);
    unsafe {
        asm!(
            // 8 comes from size of usize; 976 comes from caps_or_badges offset
            "movq $0, %gs:976(,$1,8)"
            : /* no outputs */
            : "r" (cptr),
            "r" (i)
            : "memory"
            : "volatile"
        );
    }
}

const MSG_LEN: usize = 120;

#[cfg(target_arch = "x86_64")]
pub fn sel4_set_mr(i: u32, mr: usize) {
    assert!((i as usize) < MSG_LEN);
    unsafe {
        asm!(
            // latter 8 comes from size of usize; earlier 8 comes from msg offset
            "movq $0, %gs:8(,$1,8)"
            : /* no outputs */
            : "r" (mr),
            "r" (i)
            : "memory"
            : "volatile"
        );
    }
}

#[cfg(target_arch = "x86_64")]
pub fn sel4_get_mr(i: u32) -> usize {
    assert!((i as usize) < MSG_LEN);
    let mr_out;
    unsafe {
        asm!(
            // latter 8 comes from size of usize; earlier 8 comes from msg offset
            "movq %gs:8(,$1,8), $0"
            : "=r" (mr_out)
            : "r" (i)
            : "memory"
            : "volatile"
        );
    }
    mr_out
}

// errors

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum KError {
    // TODO: make sure this stays synchronized
    NoError = 0,
    InvalidArgument = 1,
    InvalidCapability = 2,
    IllegalOperation = 3,
    RangeError = 4,
    AlignmentError = 5,
    FailedLookup = 6,
    TruncatedMessage = 7,
    DeleteFirst = 8,
    RevokeFirst = 9,
    NotEnoughMemory = 10,
    UnknownError = 11,
    // aka NumErrors
}

impl KError {
    pub fn is_error(&self) -> bool {
        self != &KError::NoError
    }

    pub fn is_okay(&self) -> bool {
        self == &KError::NoError
    }
}

const SEL4_ERRORMAP: [KError; KError::UnknownError as usize] = [
    KError::NoError,
    KError::InvalidArgument,
    KError::InvalidCapability,
    KError::IllegalOperation,
    KError::RangeError,
    KError::AlignmentError,
    KError::FailedLookup,
    KError::TruncatedMessage,
    KError::DeleteFirst,
    KError::RevokeFirst,
    KError::NotEnoughMemory];

pub fn sel4_error_for_code(code: usize) -> KError {
    if code >= SEL4_ERRORMAP.len() {
        KError::UnknownError
    } else {
        SEL4_ERRORMAP[code]
    }
}

// messageinfo

pub type MessageInfo = u32;

#[cfg(target_arch = "x86_64")]
pub fn sel4_messageinfo_new(label: u32, caps_unwrapped: u8, extra_caps: u8, length: u8) -> MessageInfo {
    /* fail if user has passed bits that we will override */
    assert!((label & !0xfffffu32) == 0);
    assert!((caps_unwrapped & !0x7u8) == 0);
    assert!((extra_caps & !0x3u8) == 0);
    assert!((length & !0x7fu8) == 0);

    (((label as u32) & 0xfffffu32) << 12) | (((caps_unwrapped as u32) & 0x7u32) << 9)
        | (((extra_caps as u32) & 0x3u32) << 7) | ((length as u32) & 0x7fu32)
}

#[cfg(target_arch = "x86_64")]
pub fn sel4_messageinfo_get_label(info: MessageInfo) -> u32 {
    (info & 0xfffff000u32) >> 12
}

// syscalls

pub fn sel4_debug_put_char(c: u8) {
    unsafe {
        x64_sys_send_recv(libsel4::seL4_Syscall_ID_seL4_SysDebugPutChar, c as u64, 0, 0, 0, 0, 0);
    }
}

pub unsafe fn sel4_call_with_mrs(dest: usize, msginfo: MessageInfo, mr0: usize, mr1: usize,
                                 mr2: usize, mr3: usize) -> (MessageInfo, usize, usize, usize, usize) {
    let (_, info_out, mr0_out, mr1_out, mr2_out, mr3_out) =
        x64_sys_send_recv(libsel4::seL4_Syscall_ID_seL4_SysCall, dest as u64, msginfo as u64,
                          mr0 as u64, mr1 as u64, mr2 as u64, mr3 as u64);

    (info_out as MessageInfo, mr0_out as usize, mr1_out as usize, mr2_out as usize, mr3_out as usize)
}

struct DebugOutput;

impl ::core::fmt::Write for DebugOutput {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for c in s.bytes() {
            sel4_debug_put_char(c);
        }
        Ok(())
    }
}

static mut STDOUT: DebugOutput = DebugOutput {};

pub fn out() -> &'static mut ::core::fmt::Write {
    unsafe {
        &mut STDOUT
    }
}

pub type seL4_BootInfo = libsel4::seL4_BootInfo;
pub type seL4_SlotRegion = libsel4::seL4_SlotRegion;
pub type seL4_UntypedDesc = libsel4::seL4_UntypedDesc;

#[no_mangle]
pub extern fn rust_main(bootinfo_addr: usize, executable_start: usize) {
    let bootinfo = unsafe {
        (bootinfo_addr as *const libsel4::seL4_BootInfo).as_ref().unwrap()
    };
    ::boot::set_bootinfo(bootinfo, executable_start);
    ::main();
    panic!("returned from main!");
}

#[lang = "eh_personality"]
extern fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(fmt: ::core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    if write!(out(), "Panic at {}:{}: {}\n", file, line, fmt).is_err() {
        for c in "Panic then panic!\n".bytes() {
            sel4_debug_put_char(c);
        }
    }
    loop {}
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    panic!("cannot unwind");
}
