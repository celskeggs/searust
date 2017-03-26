#![feature(lang_items)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(drop_types_in_const)]
#![no_std]

pub mod sel4;
mod device;
mod memory;

use core::fmt::Write;

pub fn print_bootinfo(writer: &mut Write) -> core::fmt::Result {
	let bi = sel4::sel4_bootinfo();
	try!(writeln!(writer, "BootInfo:"));
	try!(writeln!(writer, "  nodeID = {}", bi.nodeID));
	try!(writeln!(writer, "  numNodes = {}", bi.numNodes));
	try!(writeln!(writer, "  numIOPTLevels = {}", bi.numIOPTLevels as i64));
	try!(writeln!(writer, "  ipcBuffer = <object>"));
	try!(writeln!(writer, "  empty = {}", bi.empty));
	try!(writeln!(writer, "  sharedFrames = {}", bi.sharedFrames));
	try!(writeln!(writer, "  userImageFrames = {}", bi.userImageFrames));
	try!(writeln!(writer, "  userImagePaging = {}", bi.userImagePaging));
	try!(writeln!(writer, "  untyped = {}", bi.untyped));
	try!(writeln!(writer, "  untypedList = {{{}}}", bi.untyped.end - bi.untyped.start));
	try!(writeln!(writer, "  initThreadCNodeSizeBits = {}", bi.initThreadCNodeSizeBits));
	writeln!(writer, "  initThreadDomain = {}", bi.initThreadDomain)
}

pub fn main() {
	print_bootinfo(sel4::out()).unwrap();
	device::init();
}

