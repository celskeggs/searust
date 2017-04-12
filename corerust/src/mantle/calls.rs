use mantle::kernel;
use mantle::kernel::KError;
use mantle::kio;

fn handle_err(outputs: (u32, usize, usize, usize, usize), quiet: bool) -> KError {
    let result: KError = KError::from_code(kernel::messageinfo_get_label(outputs.0 as kernel::MessageInfo));
    if !quiet {
        match result {
            KError::NoError => debugc!(" --> success"),
            KError::IllegalOperation => debugc!("\n    --> illegal operation"),
            KError::FailedLookup => {
                let is_source = outputs.1;
                assert!(is_source == 0 || is_source == 1);
                let source_dest = if is_source == 0 { "destination" } else { "source" };
                let lookup_failure_type = outputs.2;
                match lookup_failure_type {
                    kernel::LOOKUP_FAILURE_INVALID_ROOT =>
                        debugc!("\n    --> failed to lookup {} cap: InvalidRoot", source_dest),
                    kernel::LOOKUP_FAILURE_MISSING_CAPABILITY =>
                        debugc!("\n    --> failed to lookup {} cap: MissingCapability with {} bits left", source_dest, outputs.3),
                    kernel::LOOKUP_FAILURE_DEPTH_MISMATCH =>
                        debugc!("\n    --> failed to lookup {} cap: DepthMismatch with {} bits left and {} bits resolved", source_dest, outputs.3, outputs.4),
                    kernel::LOOKUP_FAILURE_GUARD_MISMATCH =>
                        debugc!("\n    --> failed to lookup {} cap: GuardMismatch with {} bits left, guard {}, and {} bits of guard", source_dest, outputs.3, outputs.4, kio::get_mr(4)),
                    _ =>
                        debugc!("\n    --> failed to lookup {} cap: unexplicated variant {}", source_dest, outputs.2)
                }
            },
            _ => debugc!("\n    --> unexplicated error {:?}", result)
        };
    }
    result
}

unsafe fn call_0(service: usize, label: u32, caps: u8) -> KError {
    let tag = kernel::messageinfo_new(label, 0, caps, 0);
    handle_err(kio::call_with_mrs(service, tag, 0, 0, 0, 0), false)
}

unsafe fn call_1(service: usize, label: u32, caps: u8, mr0: usize) -> KError {
    let tag = kernel::messageinfo_new(label, 0, caps, 2);
    handle_err(kio::call_with_mrs(service, tag, mr0, 0, 0, 0), false)
}

unsafe fn call_1o(service: usize, label: u32, caps: u8, mr0: usize) -> (KError, usize, usize, usize, usize) {
    let tag = kernel::messageinfo_new(label, 0, caps, 2);
    let outputs = kio::call_with_mrs(service, tag, mr0, 0, 0, 0);
    (handle_err(outputs, label == kernel::TAG_X86_IOPORT_IN8), outputs.1, outputs.2, outputs.3, outputs.4)
}

unsafe fn call_2(service: usize, label: u32, caps: u8, mr0: usize, mr1: usize) -> KError {
    let tag = kernel::messageinfo_new(label, 0, caps, 2);
    handle_err(kio::call_with_mrs(service, tag, mr0, mr1, 0, 0), label == kernel::TAG_X86_IOPORT_OUT8)
}

unsafe fn call_3(service: usize, label: u32, caps: u8, mr0: usize, mr1: usize, mr2: usize) -> KError {
    let tag = kernel::messageinfo_new(label, 0, caps, 3);
    handle_err(kio::call_with_mrs(service, tag, mr0, mr1, mr2, 0), false)
}

unsafe fn call_4(service: usize, label: u32, caps: u8, mr0: usize, mr1: usize, mr2: usize, mr3: usize) -> KError {
    let tag = kernel::messageinfo_new(label, 0, caps, 4);
    handle_err(kio::call_with_mrs(service, tag, mr0, mr1, mr2, mr3), false)
}

unsafe fn call_6(service: usize, label: u32, caps: u8, mr0: usize, mr1: usize, mr2: usize, mr3: usize, mr4: usize, mr5: usize) -> KError {
    let tag = kernel::messageinfo_new(label, 0, caps, 6);
    kio::set_mr(4, mr4);
    kio::set_mr(5, mr5);
    handle_err(kio::call_with_mrs(service, tag, mr0, mr1, mr2, mr3), false)
}

pub fn untyped_retype(service: usize, objtype: usize, size_bits: usize, root: usize,
                           node_index: usize, node_depth: usize, node_offset: usize, num_objects: usize) -> KError {
    debugnl!("performing untyped_retype(service={}, objtype={}, size_bits={}, root={}, node_index={}, node_depth={}, node_offset={}, num_objects={})",
        service, objtype, size_bits, root, node_index, node_depth, node_offset, num_objects);
    kio::set_cap(0, root);
    unsafe { call_6(service, kernel::TAG_UNTYPED_RETYPE, 1,
                    objtype, size_bits, node_index, node_depth, node_offset, num_objects) }
}

pub fn cnode_delete(service: usize, index: usize, depth: u8) -> KError {
    debugnl!("performing cnode_delete(service={}, index={}, depth={})",
        service, index, depth);
    unsafe { call_2(service, kernel::TAG_CNODE_DELETE, 0, index, depth as usize )}
}

pub fn x86_page_map(service: usize, vroot: usize, vaddr: usize, rights: usize, vmattrs: usize) -> KError {
    debugnl!("performing x86_page_map(service={}, vroot={}, vaddr={:#X}, rights={}, vmattrs={})",
        service, vroot, vaddr, rights, vmattrs);
    kio::set_cap(0, vroot);
    unsafe { call_3(service, kernel::TAG_X86_PAGE_MAP, 1, vaddr, rights, vmattrs) }
}

pub fn x86_page_unmap(service: usize) -> KError {
    debugnl!("performing x86_page_unmap(service={})", service);
    unsafe { call_0(service, kernel::TAG_X86_PAGE_UNMAP, 0)}
}

pub fn x86_page_table_map(service: usize, vroot: usize, vaddr: usize, vmattrs: usize) -> KError {
    debugnl!("performing x86_page_table_map(service={}, vroot={}, vaddr={:#X}, vmattrs={})",
        service, vroot, vaddr, vmattrs);
    kio::set_cap(0, vroot);
    unsafe { call_2(service, kernel::TAG_X86_PAGETABLE_MAP, 1, vaddr, vmattrs) }
}

pub fn x86_page_table_unmap(service: usize) -> KError {
    debugnl!("performing x86_page_table_unmap(service={})", service);
    unsafe { call_0(service, kernel::TAG_X86_PAGETABLE_UNMAP, 0)}
}

pub fn x86_ioport_in8(service: usize, port: u16) -> (KError, u8) {
    //debugnl!("performing x86_ioport_in8(service={}, port={})", service, port);
    let out = unsafe { call_1o(service, kernel::TAG_X86_IOPORT_IN8, 0, port as usize) };
    (out.0, out.1 as u8)
}

pub fn x86_ioport_out8(service: usize, port: u16, data: u8) -> KError {
    //debugnl!("performing x86_ioport_out8(service={}, port={}, data={})", service, port, data);
    unsafe { call_2(service, kernel::TAG_X86_IOPORT_OUT8, 0, port as usize, data as usize) }
}
