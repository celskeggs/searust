use ::objs;
use ::memory;
use ::core;
use ::sel4::KError;
use ::core::cell::RefCell;
use ::core::cell::RefMut;
use ::concurrency::SingleThreaded;

pub struct VRegion {
    // both page-aligned
    start: usize,
    end: usize
}

impl VRegion {
    fn new(start: usize, end: usize) -> VRegion {
        assert!((start & (objs::PAGE_4K_SIZE - 1)) == 0);
        assert!((end & (objs::PAGE_4K_SIZE - 1)) == 0);
        assert!(end > start);
        VRegion { start, end }
    }

    pub fn len(&self) -> usize {
        assert!(self.end >= self.start);
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn chop_len(&mut self, length: usize) -> VRegion {
        assert!((length & (objs::PAGE_4K_SIZE - 1)) == 0 && length > 0);
        assert!(self.len() >= length);
        let out = VRegion::new(self.start, self.start + length);
        self.start += length;
        out
    }

    pub fn intersection(&self, other: &VRegion) -> Option<VRegion> {
        let (lower, higher) = if self.start < other.start {
            (self, other)
        } else {
            (other, self)
        };
        if lower.end > higher.start {
            Some(VRegion { start: higher.start, end: lower.end })
        } else {
            None
        }
    }

    fn join(&mut self, other: VRegion) -> Option<VRegion> {
        assert!(self.intersection(&other).is_none()); // intersections are BAD
        if self.end == other.start {
            self.end = other.end;
            None
        } else if self.start == other.end {
            self.start = other.start;
            None
        } else {
            Some(other)
        }
    }

    pub fn could_join(&self, other: &VRegion) -> bool {
        assert!(self.intersection(&other).is_none()); // intersections are BAD
        self.end == other.start || self.start == other.end
    }

    pub fn to_4k_address(&self) -> usize {
        assert!((self.start & (objs::PAGE_4K_SIZE - 1)) == 0);
        assert!((self.end - self.start) == objs::PAGE_4K_SIZE);
        self.start
    }
}

impl core::fmt::Display for VRegion {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:#X}-{:#X}", self.start, self.end)
    }
}

#[cfg(target_arch = "x86")]
const KERNEL_BASE_VADDR: usize = 0xe0000000usize;
#[cfg(target_arch = "x86_64")]
const KERNEL_BASE_VADDR: usize = 0xffffffff80000000usize;

static AVAILABLE_REGIONS: SingleThreaded<RefCell<memory::LinkedList<VRegion>>> = SingleThreaded(RefCell::new(memory::LinkedList::empty()));

fn get_avail_regions_list() -> RefMut<'static, memory::LinkedList<VRegion>> {
    AVAILABLE_REGIONS.get().borrow_mut()
}

pub fn init_vspace(executable_start: usize, image_len: usize) {
    let region = &mut *get_avail_regions_list();
    region.pushmut(VRegion::new(executable_start + image_len + objs::PAGE_4K_SIZE * 8, KERNEL_BASE_VADDR));
    debug!("TODO: readd low-memory region");
    // region.pushmut(VRegion::new(objs::PAGE_2M_SIZE, executable_start));
    debug!("self was loaded to: {:#X}-{:#X}", executable_start, executable_start + image_len);
}

pub fn allocate_vregion(length: usize) -> core::result::Result<VRegion, KError> {
    assert!((length & (objs::PAGE_4K_SIZE - 1)) == 0 && length > 0);
    let rl: &mut memory::LinkedList<VRegion> = &mut *get_avail_regions_list();
    let (vregion, is_now_empty): (VRegion, bool) = {
        let h = rl.find_mut(|b| b.len() >= length);
        if h.is_none() {
            return Err(KError::NotEnoughMemory);
        }
        let head = h.unwrap();
        (head.chop_len(length), head.is_empty())
    };
    if is_now_empty {
        assert!(rl.remove_mut(|b| b.is_empty()).unwrap().is_empty());
        assert!(rl.find(|b| b.is_empty()).is_none());
    }
    debug!("allocated vregion {}", vregion);
    Ok(vregion)
}

pub fn free_vregion(mut r: VRegion) {
    assert!(!r.is_empty());
    let rl: &mut memory::LinkedList<VRegion> = &mut *get_avail_regions_list();
    let mut cur: &mut memory::LinkedList<VRegion> = rl;
    loop {
        let tmp = cur;
        if tmp.is_empty() {
            // nope -- cur is the end of the line! just add our stuff.
            if tmp.pushmut(r).is_err() {
                panic!("could not free vregion due to OOM condition");
            }
            // added!
            return;
        }
        let (head, ncur) = tmp.nextmut().unwrap();
        // try to merge this one
        if let Some(rest) = head.join(r) {
            // we can't merge here. we'll try the next element
            r = rest;
        } else {
            // merged! hooray! now we need to see if we can merge onto the next one as well
            let should_do_postjoin =
                if let Some(adjacent) = ncur.head() {
                    head.could_join(adjacent)
                } else {
                    false
                };
            if should_do_postjoin {
                assert!(head.join(ncur.popmut().unwrap()).is_none()); // make sure we're successful
            }
            return;
        }
        cur = ncur
    }
}
