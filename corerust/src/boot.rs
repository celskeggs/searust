pub struct BootInfo {}

pub fn print_bootinfo(writer: &mut ::core::fmt::Write, bi: &::sel4::seL4_BootInfo) -> ::core::fmt::Result {
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

pub fn set_bootinfo(bi: &::sel4::seL4_BootInfo, executable_start: usize) {
    print_bootinfo(::sel4::out(), bi).unwrap();
    ::device::init_untyped(::caps::CapRange::range(bi.untyped.start as usize, bi.untyped.end as usize), bi.untypedList);
    ::caps::init_cslots(::caps::CapRange::range(bi.empty.start as usize, bi.empty.end as usize));
    let image_len = ((bi.userImageFrames.end - bi.userImageFrames.start) as usize) * ::objs::PAGE_4K_SIZE;
    ::vspace::init_vspace(executable_start, image_len);
}
