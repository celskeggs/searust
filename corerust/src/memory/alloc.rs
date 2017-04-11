use ::core;

const TRACE: bool = false;

mod fixed_alloc {
    const HEAP_KB: usize = 64;
    const HEAP_U64: usize = HEAP_KB * (1024 / 8);
    static mut EARLY_HEAP: [u64; HEAP_U64] = [0; HEAP_U64]; // start with 64KB of memory
    static mut FIRST_FREE: usize = 0;

    pub fn alloc_fixblks(size: u8) -> Option<*mut u64> {
        assert!(size != 0);
        unsafe {
            if FIRST_FREE + (size as usize) <= HEAP_U64 {
                let nptr = &mut EARLY_HEAP[FIRST_FREE] as *mut u64;
                FIRST_FREE += size as usize;
                if super::TRACE {
                    debug!("allocated {} blocks from unallocated memory --> {}/{}", size, FIRST_FREE, HEAP_U64);
                }
                Some(nptr)
            } else {
                None
            }
        }
    }
}

mod dynamic_alloc {
    use ::core;
    use ::crust;
    use ::crust::vspace::VRegion;
    use ::kobject::*;
    use ::mantle::KError;
    use ::mantle::kernel;
    use ::memory::LinkedList;
    use ::memory::untyped;

    struct DynamicAllocator {
        vregion: VRegion,
        next_avail: usize,
        next_unalloc: usize,
        pages: LinkedList<FixedMappedPage4K>,
        is_recursing: bool
    }

    const VREGION_BITS: u8 = 28; // 256 MB
    const ALLOCATION_BUFFER: usize = kernel::PAGE_4K_SIZE * 8;

    impl DynamicAllocator {
        fn new() -> core::result::Result<DynamicAllocator, KError> {
            let vregion = crust::vspace::allocate_vregion(1 << VREGION_BITS)?;
            // basic sanity checking so we can make these assumptions later
            assert!(vregion.start() & (kernel::PAGE_4K_SIZE - 1) == 0);
            assert!(vregion.len() == (1 << VREGION_BITS));
            Ok(DynamicAllocator { vregion, next_avail: 0, next_unalloc: 0, pages: LinkedList::empty(), is_recursing: false })
        }

        fn add_fresh_page(&mut self) -> core::result::Result<(), KError> {
            if self.next_unalloc + kernel::PAGE_4K_SIZE > self.vregion.len() {
                Err(KError::NotEnoughMemory)
            } else {
                let page = untyped::allocate_page4k()?;
                match page.map_into_addr(self.next_unalloc + self.vregion.start(), true) {
                    Ok(mapping) => {
                        self.next_unalloc += kernel::PAGE_4K_SIZE;
                        if let Err(_) = self.pages.pushmut(mapping) {
                            panic!("could not allocate memory to save new memory mapping");
                        }
                        Ok(())
                    },
                    Err((page, err)) => {
                        untyped::free_page4k(page);
                        Err(err)
                    }
                }
            }
        }

        // ensure that we're at least to this point
        fn alloc_forward(&mut self, min_unalloc: usize) -> core::result::Result<(), KError> {
            if self.is_recursing {
                if self.next_unalloc >= min_unalloc { // well... good enough for now?
                    Ok(())
                } else {
                    panic!("recursive dynamic memory allocation!");
                }
            } else {
                self.is_recursing = true;
                // we include ALLOCATION_BUFFER here so that we always try to have extra room
                while self.next_unalloc < min_unalloc + ALLOCATION_BUFFER {
                    if let Err(err) = self.add_fresh_page() {
                        self.is_recursing = false;
                        if self.next_unalloc >= min_unalloc { // well... good enough for now?
                            return Ok(());
                        } else {
                            return Err(err);
                        }
                    }
                }
                self.is_recursing = false;
                Ok(())
            }
        }

        fn alloc_fixblks(&mut self, size: u8) -> Option<*mut u64> {
            let real_size = (size as usize) * 8;
            let until = self.next_avail + real_size;
            if let Err(err) = self.alloc_forward(until) {
                debug!("Could not allocate additional dynamic memory: {:?}", err);
                return None;
            }
            let ptr = (self.next_avail + self.vregion.start()) as *mut u64;
            self.next_avail += real_size;
            if super::TRACE {
                debug!("allocated {} blocks from unallocated memory --> {}/{}/{}", size, self.next_avail / 8, self.next_unalloc / 8, self.vregion.len() / 8);
            }
            Some(ptr)
        }
    }

    static mut ALLOC: Option<DynamicAllocator> = None;

    pub fn init() {
        let alloc = Some(DynamicAllocator::new().expect("could not init dynamic allocator"));
        unsafe {
            assert!(ALLOC.is_none());
            ALLOC = alloc;
        }
    }

    pub fn alloc_fixblks(size: u8) -> Option<*mut u64> {
        if let &mut Some(ref mut dyna) = unsafe { &mut ALLOC } {
            dyna.alloc_fixblks(size)
        } else {
            None
        }
    }
}

mod recycle_alloc {
    // each bucket is a multiple of 8 bytes. the maximum fixblk is 255 * 8 bytes == 2040 bytes, so we have 255 buckets.
    static mut BUCKETS: [*mut u64; 255] = [::core::ptr::null_mut(); 255];

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
                if super::TRACE {
                    debug!("allocated {} blocks from recycled memory", size);
                }
                Some(ptr)
            } else {
                None
            }
        }
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

pub fn alloc_fix(size: u16) -> Option<*mut u64> {
    assert!(size >= 1 && size <= 255 * 8);
    let blks = ((size + 7) / 8) as u8;
    recycle_alloc::alloc_fixblks(blks).or_else(|| fixed_alloc::alloc_fixblks(blks)).or_else(|| dynamic_alloc::alloc_fixblks(blks))
}

pub fn alloc_type<T>(x: T) -> core::result::Result<*mut T, T> {
    let size: usize = core::mem::size_of::<T>();
    assert!(size < 65536);
    if let Some(ptr) = alloc_fix(size as u16) {
        let cptr = ptr as *mut T;
        unsafe {
            core::ptr::write(cptr, x);
        }
        Ok(cptr)
    } else {
        debug!("failed to allocate {} bytes", size);
        Err(x)
    }
}

pub unsafe fn dealloc_type<T>(ptr: *mut T) -> T {
    assert!(!ptr.is_null());
    let out = core::ptr::read(ptr);
    // TODO: zero out first?
    let size: usize = core::mem::size_of::<T>();
    assert!(size < 65536);
    if TRACE {
        debug!("deallocated {} bytes", size);
    }
    recycle_alloc::dealloc_fix(ptr as *mut u64, size as u16);
    out
}

pub fn init_allocator() {
    dynamic_alloc::init();
}