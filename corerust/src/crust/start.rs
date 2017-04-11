use ::mantle;
use ::memory;
use mantle::kernel;
use mantle::kernel::BootInfo;
use ::crust;
use ::core;
use ::kobject::*;

#[no_mangle]
pub extern fn mantle_main(bootinfo: &BootInfo, executable_start: usize) {
    set_bootinfo(bootinfo, executable_start);
    ::main(bootinfo);
}

pub fn print_bootinfo(writer: &mut core::fmt::Write, bi: &BootInfo) -> core::fmt::Result {
    try!(writeln!(writer, "BootInfo:"));
    try!(writeln!(writer, "  nodeID = {}", bi.node_id));
    try!(writeln!(writer, "  numNodes = {}", bi.num_nodes));
    try!(writeln!(writer, "  numIOPTLevels = {}", bi.num_iopt_levels as i64));
    try!(writeln!(writer, "  ipcBuffer = <object>"));
    try!(writeln!(writer, "  empty = {}", bi.empty));
    try!(writeln!(writer, "  sharedFrames = {}", bi.shared_frames));
    try!(writeln!(writer, "  userImageFrames = {}", bi.user_image_frames));
    try!(writeln!(writer, "  userImagePaging = {}", bi.user_image_paging));
    try!(writeln!(writer, "  untyped = {}", bi.untyped));
    try!(writeln!(writer, "  untypedList = {{{}}}", bi.untyped.end - bi.untyped.start));
    try!(writeln!(writer, "  initThreadCNodeSizeBits = {}", bi.init_thread_cnode_size_bits));
    writeln!(writer, "  initThreadDomain = {}", bi.init_thread_domain)
}

fn set_bootinfo(bi: &BootInfo, executable_start: usize) {
    let image_len = ((bi.user_image_frames.end - bi.user_image_frames.start) as usize) * kernel::PAGE_4K_SIZE;
    print_bootinfo(mantle::debug(), bi).unwrap();
    crust::capalloc::init_cslots(CapRange::range(bi.empty.start as usize, bi.empty.end as usize));
    crust::vspace::init_vspace(executable_start, image_len);
    memory::init_allocator();
    memory::untyped::init_untyped(CapRange::range(bi.untyped.start as usize, bi.untyped.end as usize), bi.untyped_list);
    memory::device::init_untyped(CapRange::range(bi.untyped.start as usize, bi.untyped.end as usize), bi.untyped_list);
}
