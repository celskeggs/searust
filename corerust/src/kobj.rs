
use ::sel4::*;

pub fn sel4_untyped_retype(service: usize, objtype: usize, size_bits: usize, root: usize,
        node_index: usize, node_depth: usize, node_offset: usize, num_objects: usize) -> KError {
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

    if result != KError::NoError {
        sel4_set_mr(0, outputs.1);
        sel4_set_mr(1, outputs.2);
        sel4_set_mr(2, outputs.3);
        sel4_set_mr(3, outputs.4);
    }

    result
}

pub fn sel4_cnode_delete(service: usize, index: usize, depth: u8) -> KError {
    let tag = sel4_messageinfo_new(libsel4::invocation_label_CNodeDelete, 0, 0, 2);
    let mr0 = index;
    let mr1 = (depth as usize) & 0xff;

    let outputs = unsafe {
        sel4_call_with_mrs(service, tag, mr0, mr1, 0, 0)
    };
    let output_tag = outputs.0;
    let result: KError = sel4_error_for_code(sel4_messageinfo_get_label(output_tag) as usize);

    if result != KError::NoError {
        sel4_set_mr(0, outputs.1);
        sel4_set_mr(1, outputs.2);
        sel4_set_mr(2, outputs.3);
        sel4_set_mr(3, outputs.4);
    }

    result
}
