#![feature(lang_items)]
#![feature(asm)]
#![no_std]

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

#[cfg(target_arch = "x86_64")]
fn x64_sys_send_recv(syscall: i64, dest: u64, info: u64, mr0: u64, mr1: u64, mr2: u64, mr3: u64) -> [u64; 6] {
	let info_out;
	let dest_out;
	let mr0_out;
	let mr1_out;
	let mr2_out;
	let mr3_out;
	unsafe {
		asm!(
			"movq %rsp, %rcx\nleaq 1f, %rdx\n1: sysenter\n"
			: "={rsi}"(info_out),
			  "={r10}"(mr0_out),
			  "={r8}"(mr1_out),
			  "={r9}"(mr2_out),
			  "={r15}"(mr3_out),
			  "={rdi}"(dest_out)
			: "{rax}"(syscall),
			  "{rdi}"(dest),
			  "{rsi}"(info),
			  "{r10}"(mr0),
			  "{r8}"(mr1),
			  "{r9}"(mr2),
			  "{r15}"(mr3)
			: "%rcx", "%rdx", "memory" 
			: "volatile"
		);
	}
	return [dest_out, info_out, mr0_out, mr1_out, mr2_out, mr3_out];
}

fn sel4_debug_put_char(c : char) {
	x64_sys_send_recv(libsel4::seL4_Syscall_ID_seL4_SysDebugPutChar, c as u64, 0, 0, 0, 0, 0);
	return;
}

#[no_mangle]
pub extern fn rust_main() {
	sel4_debug_put_char('H');
	sel4_debug_put_char('e');
	sel4_debug_put_char('l');
	sel4_debug_put_char('l');
	sel4_debug_put_char('o');
	sel4_debug_put_char('\n');
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] #[no_mangle] pub extern fn panic_fmt() -> ! {loop{}}
