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

mod libsel4 {
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

#[cfg(target_arch = "x86_64")]
unsafe fn x64_sys_send_recv(syscall: i64, dest: u64, info: u64, mr0: u64, mr1: u64, mr2: u64, mr3: u64) -> [u64; 6] {
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
	[dest_out, info_out, mr0_out, mr1_out, mr2_out, mr3_out]
}

pub fn sel4_debug_put_char(c : u8) {
	unsafe {
		x64_sys_send_recv(libsel4::seL4_Syscall_ID_seL4_SysDebugPutChar, c as u64, 0, 0, 0, 0, 0);
	}
}

struct DebugOutput {
}

impl ::core::fmt::Write for DebugOutput {
	fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
		for c in s.bytes() {
			sel4_debug_put_char(c);
		}
		Ok(())
	}
}

static mut BOOTINFO: Option<&libsel4::seL4_BootInfo> = None;
static mut STDOUT: DebugOutput = DebugOutput {};

pub fn out() -> &'static mut ::core::fmt::Write {
	unsafe {
		&mut STDOUT
	}
}

pub fn sel4_bootinfo() -> &'static libsel4::seL4_BootInfo {
	unsafe {
		BOOTINFO.unwrap()
	}
}

#[no_mangle]
pub extern fn rust_main(bootinfo_addr: usize) {
	writeln!(out(), "Bootinfo address: {}", bootinfo_addr);
	unsafe {
		BOOTINFO = Some((bootinfo_addr as *const libsel4::seL4_BootInfo).as_ref().unwrap());
	}
	::main();
	panic!("returned from main!");
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(fmt: ::core::fmt::Arguments, file: &'static str, line: u32) -> ! {
	if write!(out(), "Panic at {}:{}: {}\n", file, line, fmt).is_err() {
		for c in "Panic then panic!\n".bytes() {
			sel4_debug_put_char(c);
		}
	}
	loop{ }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    panic!("cannot unwind");
}

