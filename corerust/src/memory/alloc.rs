use ::core;

mod alloc {
    // MEMORY ALLOCATION MACHINERY
    const HEAP_KB: usize = 64;
    const HEAP_U64: usize = HEAP_KB * (1024 / 8);

    // each bucket is a multiple of 8 bytes. the maximum fixblk is 255 * 8 bytes == 2040 bytes, so we have 255 buckets.
    static mut EARLY_HEAP: [u64; HEAP_U64] = [0; HEAP_U64]; // start with 64KB of memory
    static mut BUCKETS: [*mut u64; 255] = [::core::ptr::null_mut(); 255];
    static mut FIRST_FREE: usize = 0;

    unsafe fn deref_seq(ptr: *mut u64) -> Option<*mut u64> {
        if ptr.is_null() {
            None
        } else {
            let target_addr: u64 = *ptr;
            *ptr = 0;
            Some(target_addr as *mut u64)
        }
    }

    unsafe fn reref_seq(ptr: *mut u64, older: *mut u64) -> *mut u64 {
        assert!(!ptr.is_null());
        *ptr = older as u64;
        ptr
    }

    pub fn alloc_fixblks(size: u8) -> Option<*mut u64> {
        assert!(size != 0);
        unsafe {
            let ptr = BUCKETS[(size - 1) as usize];
            if let Some(nptr) = deref_seq(ptr) {
                BUCKETS[(size - 1) as usize] = nptr;
                Some(ptr)
            } else if FIRST_FREE + (size as usize) <= HEAP_U64 {
                let nptr = &mut EARLY_HEAP[FIRST_FREE] as *mut u64;
                FIRST_FREE += size as usize;
                Some(nptr)
            } else {
                None
            }
        }
    }

    pub fn alloc_fix(size: u16) -> Option<*mut u64> {
        assert!(size >= 1 && size <= 255 * 8);
        alloc_fixblks(((size + 7) / 8) as u8)
    }

    pub unsafe fn dealloc_fixblks(ptr: *mut u64, size: u8) {
        assert!(size != 0);
        BUCKETS[(size - 1) as usize] = reref_seq(ptr, BUCKETS[(size - 1) as usize]);
    }

    pub unsafe fn dealloc_fix(ptr: *mut u64, size: u16) {
        assert!(size >= 1 && size <= 255 * 8);
        dealloc_fixblks(ptr, ((size + 7) / 8) as u8)
    }
}

pub fn alloc_type<T>(x: T) -> core::result::Result<*mut T, T> {
    let size: usize = core::mem::size_of::<T>();
    assert!(size < 65536);
    if let Some(ptr) = alloc::alloc_fix(size as u16) {
        let cptr = ptr as *mut T;
        unsafe {
            core::ptr::write(cptr, x);
        }
        Ok(cptr)
    } else {
        Err(x)
    }
}

pub unsafe fn dealloc_type<T>(ptr: *mut T) -> T {
    assert!(!ptr.is_null());
    let out = core::ptr::read(ptr);
    // TODO: zero out first?
    let size: usize = core::mem::size_of::<T>();
    assert!(size < 65536);
    alloc::dealloc_fix(ptr as *mut u64, size as u16);
    out
}
