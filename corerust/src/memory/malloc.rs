use ::memory;
use ::core;

// TODO: don't base this off of the slab allocator......

pub const MAX_VARALLOC_LEN: usize = memory::alloc::MAX_ALLOC_LEN - 8;
const VARALLOC_MAGIC: u64 = 0x737A200737A20000;
const VARALLOC_MAGIC_MASK: u64 = 0xFFFFFFFFFFFFF800;

pub fn malloc(len: usize) -> Option<*mut u8> {
    if len <= MAX_VARALLOC_LEN {
        let total_len = (len + 8) as u64; // 8 for usize for tracking real length
        assert!((total_len & VARALLOC_MAGIC_MASK) == 0);
        if let Some(ptr) = memory::alloc::alloc_fix(total_len as u16) {
            unsafe {
                *ptr = total_len | VARALLOC_MAGIC;
                Some((ptr as *mut u8).offset(8))
            }
        } else {
            None
        }
    } else {
        panic!("not yet implemented: allocations larger than {} bytes", MAX_VARALLOC_LEN);
    }
}

pub unsafe fn get_len(ptr: *mut u8) -> usize {
    assert!(!ptr.is_null());
    let realptr = (ptr.offset(-8)) as *mut u64;
    let len = *realptr;
    assert!((len & VARALLOC_MAGIC_MASK) == VARALLOC_MAGIC); // otherwise it's probably not a real allocation
    let reallen = len & !VARALLOC_MAGIC_MASK;
    assert!(reallen <= memory::alloc::MAX_ALLOC_LEN as u64);
    (reallen - 8) as usize
}

pub unsafe fn free(ptr: *mut u8) {
    memory::alloc::dealloc_fix((ptr.offset(-8)) as *mut u64, (get_len(ptr) + 8) as u16);
}

// on failure, passed-in pointer is still valid
pub unsafe fn realloc(ptr: *mut u8, len: usize) -> Option<*mut u8> {
    let min_len = core::cmp::min(len, get_len(ptr));
    if let Some(n_alloc) = malloc(len) {
        core::ptr::copy_nonoverlapping(ptr, n_alloc, min_len);
        free(ptr);
        Some(n_alloc)
    } else {
        None
    }
}
