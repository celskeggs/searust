use ::sel4::*;
use ::libsel4;

pub fn sel4_untyped_retype(service: usize, objtype: usize, size_bits: usize, root: usize,
                           node_index: usize, node_depth: usize, node_offset: usize, num_objects: usize) -> KError {
    assert!(num_objects > 0);
    let tag = sel4_messageinfo_new(libsel4::invocation_label_UntypedRetype, 0, 1, 6);
    sel4_set_cap(0, root);
    let mr0 = objtype;
    let mr1 = size_bits;
    let mr2 = node_index;
    let mr3 = node_depth;
    sel4_set_mr(4, node_offset);
    sel4_set_mr(5, num_objects);

    let outputs = unsafe {
        sel4_call_with_mrs(service, tag, mr0, mr1, mr2, mr3)
    };
    let output_tag = outputs.0;
    let result: KError = sel4_error_for_code(sel4_messageinfo_get_label(output_tag) as usize);

    debugnl!("performing sel4_untyped_retype(service={}, objtype={}, size_bits={}, root={}, node_index={}, node_depth={}, node_offset={}, num_objects={})",
        service, objtype, size_bits, root, node_index, node_depth, node_offset, num_objects);
    if result == KError::NoError {
        debugc!(" --> success");
    } else if result == KError::FailedLookup {
        let is_source = outputs.1;
        assert!(is_source == 0 || is_source == 1);
        let source_dest = if is_source == 0 {
            "destination"
        } else {
            "source"
        };
        let lookup_failure_type = outputs.2;
        match lookup_failure_type {
            libsel4::seL4_LookupFailureType_seL4_InvalidRoot => {
                debugc!("\n    --> failed to lookup {} cap: InvalidRoot", source_dest);
            }
            libsel4::seL4_LookupFailureType_seL4_MissingCapability => {
                debugc!("\n    --> failed to lookup {} cap: MissingCapability with {} bits left", source_dest, outputs.3);
            }
            libsel4::seL4_LookupFailureType_seL4_DepthMismatch => {
                debugc!("\n    --> failed to lookup {} cap: DepthMismatch with {} bits left and {} bits resolved", source_dest, outputs.3, outputs.4);
            }
            libsel4::seL4_LookupFailureType_seL4_GuardMismatch => {
                debugc!("\n    --> failed to lookup {} cap: GuardMismatch with {} bits left, guard {}, and {} bits of guard", source_dest, outputs.3, outputs.4, sel4_get_mr(4));
            }
            _ => {
                debugc!("\n    --> failed to lookup {} cap: unexplicated variant {}", source_dest, outputs.2);
            }
        }
    } else {
        debugc!("\n    --> unexplicated error {:?}", result);
        sel4_set_mr(0, outputs.1);
        sel4_set_mr(1, outputs.2);
        sel4_set_mr(2, outputs.3);
        sel4_set_mr(3, outputs.4);
    }

    result
}

pub fn sel4_cnode_delete(service: usize, index: usize, depth: u8) -> KError {
    let tag = sel4_messageinfo_new(libsel4::invocation_label_CNodeDelete, 0, 0, 2);

    let outputs = unsafe {
        sel4_call_with_mrs(service, tag, index, (depth as usize) & 0xff, 0, 0)
    };
    let output_tag = outputs.0;
    let result: KError = sel4_error_for_code(sel4_messageinfo_get_label(output_tag) as usize);
    debug!("performed sel4_cnode_delete(service={}, index={}, depth={}) -> result={:?}",
    service, index, depth, result);

    if result != KError::NoError {
        sel4_set_mr(0, outputs.1);
        sel4_set_mr(1, outputs.2);
        sel4_set_mr(2, outputs.3);
        sel4_set_mr(3, outputs.4);
    }

    result
}

pub fn sel4_x86_page_map(service: usize, vroot: usize, vaddr: usize, rights: usize, vmattrs: usize) -> KError {
    let tag = sel4_messageinfo_new(libsel4::arch_invocation_label_X86PageMap, 0, 1, 3);
    sel4_set_cap(0, vroot);

    let outputs = unsafe {
        sel4_call_with_mrs(service, tag, vaddr, rights, vmattrs, 0)
    };
    let output_tag = outputs.0;
    let result: KError = sel4_error_for_code(sel4_messageinfo_get_label(output_tag) as usize);

    debugnl!("performing sel4_x86_page_map(service={}, vroot={}, vaddr={:#X}, rights={}, vmattrs={})",
        service, vroot, vaddr, rights, vmattrs);
    if result == KError::NoError {
        debugc!(" --> success");
    } else if result == KError::IllegalOperation {
        debugc!("\n    --> illegal operation");
    } else if result == KError::FailedLookup && outputs.1 == 0 {
        debugc!("\n    --> failed lookup (probably no page table)");
    } else {
        debugc!("\n    --> unexplicated error {:?}", result);
        sel4_set_mr(0, outputs.1);
        sel4_set_mr(1, outputs.2);
        sel4_set_mr(2, outputs.3);
        sel4_set_mr(3, outputs.4);
    }

    result
}

pub fn sel4_x86_page_unmap(service: usize) -> KError {
    let tag = sel4_messageinfo_new(libsel4::arch_invocation_label_X86PageUnmap, 0, 0, 0);

    let outputs = unsafe {
        sel4_call_with_mrs(service, tag, 0, 0, 0, 0)
    };
    let output_tag = outputs.0;
    let result: KError = sel4_error_for_code(sel4_messageinfo_get_label(output_tag) as usize);

    debugnl!("performing sel4_x86_page_unmap(service={})", service);
    if result == KError::NoError {
        debugc!(" --> success");
    } else {
        debugc!("\n    --> unexplicated error {:?}", result);
        sel4_set_mr(0, outputs.1);
        sel4_set_mr(1, outputs.2);
        sel4_set_mr(2, outputs.3);
        sel4_set_mr(3, outputs.4);
    }

    result
}