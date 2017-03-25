#![feature(lang_items)]
#![feature(asm)]
#![no_std]

pub mod sel4;

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
	try!(writeln!(writer, "  untypedList = {{"));
	for i in 0 .. (bi.untyped.end - bi.untyped.start) as usize {
		let desc = bi.untypedList[i];
		try!(writeln!(writer, "  [{:2>}] = {{ paddr = {:#X}", i, desc.paddr));
		try!(writeln!(writer, "           sizeBits = {}", desc.sizeBits));
		try!(writeln!(writer, "           isDevice = {} }}", desc.isDevice != 0));
	}
	try!(writeln!(writer, "  }}"));
	try!(writeln!(writer, "  initThreadCNodeSizeBits = {}", bi.initThreadCNodeSizeBits));
	writeln!(writer, "  initThreadDomain = {}", bi.initThreadDomain)
}

pub fn main() {
	print_bootinfo(sel4::out()).unwrap();
}

