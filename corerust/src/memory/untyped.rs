use core;
use crust;
use memory::LinkedList;
use mantle;
use mantle::KError;
use kobject::*;
use mantle::kernel;

pub struct UntypedAllocator {
    small_pages: LinkedList<Untyped>,
    large_pages: LinkedList<Untyped>,
    stashed: LinkedList<UntypedSet>,
}

impl UntypedAllocator {
    pub fn add_oversize_block(&mut self, ut: Untyped) {
        if ut.size_bits() <= kernel::PAGE_2M_BITS + kernel::FAN_OUT_LIMIT_BITS {
            return self.add_huge_block(ut);
        }
        // limits us to 128 GB under current settings, but the fan-out limit can be increased in the kernel if that's a problem
        assert!(ut.size_bits() <= kernel::PAGE_2M_BITS + kernel::FAN_OUT_LIMIT_BITS * 2);
        let mut goal_split_count = ut.size_bits() - (kernel::PAGE_2M_BITS + kernel::FAN_OUT_LIMIT_BITS);
        assert!(goal_split_count <= kernel::FAN_OUT_LIMIT_BITS);
        assert!(goal_split_count > 0);
        let capset = match crust::capalloc::allocate_cap_slots(1 << goal_split_count) {
            Ok(cs) => cs,
            Err(err) => panic!("could not allocate capslots for memory branching: {:?}", err)
        };
        match ut.split(goal_split_count, capset) {
            Ok(mut uts) => {
                for i in 0 .. uts.count() {
                    self.add_huge_block(uts.take_front().unwrap())
                }
                assert!(uts.take_front().is_none());
                assert!(self.stashed.pushmut(uts).is_ok());
            }, Err((err, ut, capset)) => {
                panic!("could not split oversize untyped as part of initial memory branching: {:?}", err);
            }
        }
    }

    pub fn add_huge_block(&mut self, ut: Untyped) { // used for 2^22 to 2^29
        let mut goal_split_count = ut.size_bits() - kernel::PAGE_2M_BITS;
        assert!(goal_split_count <= kernel::FAN_OUT_LIMIT_BITS);
        assert!(goal_split_count > 0);
        let capset = match crust::capalloc::allocate_cap_slots(1 << goal_split_count) {
            Ok(cs) => cs,
            Err(err) => panic!("could not allocate capslots for memory branching: {:?}", err)
        };
        match ut.split(goal_split_count, capset) {
            Ok(mut uts) => {
                for i in 0 .. uts.count() {
                    self.add_large_page(uts.take_front().unwrap())
                }
                assert!(uts.take_front().is_none());
                assert!(self.stashed.pushmut(uts).is_ok());
            }, Err((err, ut, capset)) => {
                panic!("could not split huge untyped as part of initial memory branching: {:?}", err);
            }
        }
    }

    pub fn add_large_page(&mut self, ut: Untyped) {
        assert!(ut.size_bits() == kernel::PAGE_2M_BITS);
        assert!(self.large_pages.pushmut(ut).is_ok());
    }

    pub fn add_midsize_block(&mut self, ut: Untyped) {
        assert!(ut.size_bits() > kernel::PAGE_4K_BITS && ut.size_bits() < kernel::PAGE_2M_BITS);
        let mut goal_split_count = ut.size_bits() - kernel::PAGE_4K_BITS; // 1 - 8
        // should hopefully be configured as this:
        assert!(kernel::FAN_OUT_LIMIT_BITS >= 8);
        // should hopefully be limited like this:
        // (small: 12 bits, large: 21 bits -- largest is 20 bits which means a 256-split)
        assert!(goal_split_count <= 8 && goal_split_count > 0);
        let capset = match crust::capalloc::allocate_cap_slots(1 << goal_split_count) {
            Ok(cs) => cs,
            Err(err) => panic!("could not allocate capslots for memory branching: {:?}", err)
        };
        match ut.split(goal_split_count, capset) {
            Ok(mut uts) => {
                for i in 0 .. uts.count() {
                    self.add_small_page(uts.take_front().unwrap())
                }
                assert!(uts.take_front().is_none());
                assert!(self.stashed.pushmut(uts).is_ok());
            }, Err((err, ut, capset)) => {
                panic!("could not split small untyped as part of initial memory branching: {:?}", err);
            }
        }
    }

    pub fn add_small_page(&mut self, ut: Untyped) {
        assert!(ut.size_bits() == kernel::PAGE_4K_BITS);
        assert!(self.small_pages.pushmut(ut).is_ok());
    }

    pub fn add_initial_block(&mut self, ut: Untyped) {
        if ut.size_bits() > kernel::PAGE_2M_BITS {
            self.add_oversize_block(ut);
        } else if ut.size_bits() == kernel::PAGE_2M_BITS {
            self.add_large_page(ut);
        } else if ut.size_bits() > kernel::PAGE_4K_BITS {
            self.add_midsize_block(ut);
        } else if ut.size_bits() == kernel::PAGE_4K_BITS {
            self.add_small_page(ut);
        } else {
            panic!("unexpected block is smaller than 4K");
        }
    }

    pub fn allocate_large_page(&mut self) -> core::result::Result<Untyped, KError> {
        self.large_pages.popmut().ok_or(KError::NotEnoughMemory)
    }

    pub fn allocate_small_page(&mut self) -> core::result::Result<Untyped, KError> {
        if self.small_pages.is_empty() {
            // TODO: make this actually work in low-memory conditions, because currently it won't be able to provide enough small pages without needing memory itself

            // let's take a large page, cut it up a bit (so that we can treat it as midsize pages) and add it to the pile
            let cslots = crust::capalloc::allocate_cap_slots(2)?;
            let large_page = match self.allocate_large_page() {
                Ok(page) => page,
                Err(err) => {
                    crust::capalloc::free_cap_slots(cslots);
                    return Err(err);
                }
            };
            let mut untypeds: UntypedSet = match large_page.split(1, cslots) {
                Ok(uts) => uts,
                Err((err, ut, cslots)) => {
                    crust::capalloc::free_cap_slots(cslots);
                    self.add_large_page(ut);
                    return Err(err);
                }
            };
            self.add_midsize_block(untypeds.take_front().unwrap());
            self.add_midsize_block(untypeds.take_front().unwrap());
            assert!(!untypeds.remaining());
            assert!(self.stashed.pushmut(untypeds).is_ok());
        }
        Ok(self.small_pages.popmut().unwrap())
    }

    pub fn print_info(&self, writer: &mut core::fmt::Write) -> core::fmt::Result {
        writeln!(writer, "memory info:")?;
        writeln!(writer, "  number of large blocks: {}", self.large_pages.len())?;
        writeln!(writer, "    {} KB", self.large_pages.len() * 2048)?;
        writeln!(writer, "  number of small blocks: {}", self.small_pages.len())?;
        writeln!(writer, "    {} KB", self.small_pages.len() * 4)?;
        let all_mem = self.small_pages.len() * 4 + self.large_pages.len() * 2048;
        writeln!(writer, "  total memory: {} KB = {} MB", all_mem, all_mem >> 10)
    }
}

// ordered from lowest address to highest address
static mut ALLOCATOR: UntypedAllocator =
    UntypedAllocator {
        small_pages: LinkedList::empty(),
        large_pages: LinkedList::empty(),
        stashed: LinkedList::empty()
    };

pub fn get_allocator() -> &'static mut UntypedAllocator {
    unsafe { &mut ALLOCATOR }
}

pub fn init_untyped(untyped: CapRange, untyped_list: [kernel::UntypedDesc; 230usize]) {
    let alloc = get_allocator();
    let count = untyped.len();
    for ir in 0..count {
        let i = count - 1 - ir;
        let ent = untyped_list[i];
        if ent.is_device == 0 && ent.size_bits == kernel::PAGE_4K_BITS {
            alloc.add_initial_block(Untyped::from_cap(untyped.nth(i).assert_populated(), ent.size_bits));
        }
    }
    for ir in 0..count {
        let i = count - 1 - ir;
        let ent = untyped_list[i];
        if ent.is_device == 0 && ent.size_bits != kernel::PAGE_4K_BITS && ent.size_bits < kernel::PAGE_2M_BITS {
            alloc.add_initial_block(Untyped::from_cap(untyped.nth(i).assert_populated(), ent.size_bits));
        }
    }
    for ir in 0..count {
        let i = count - 1 - ir;
        let ent = untyped_list[i];
        if ent.is_device == 0 && ent.size_bits >= kernel::PAGE_2M_BITS {
            alloc.add_initial_block(Untyped::from_cap(untyped.nth(i).assert_populated(), ent.size_bits));
        }
    }
    alloc.print_info(mantle::debug());
}

pub fn allocate_untyped_4k() -> core::result::Result<Untyped, KError> {
    get_allocator().allocate_small_page()
}

pub fn allocate_page4k() -> core::result::Result<Page4K, KError> {
    let slot = crust::capalloc::allocate_cap_slot()?;
    let ut = match allocate_untyped_4k() {
        Ok(ut) => ut,
        Err(err) => {
            crust::capalloc::free_cap_slot(slot);
            return Err(err);
        }
    };
    match ut.become_page_4k(slot) {
        Ok(page) => Ok(page),
        Err((err, ut, cs)) => {
            crust::capalloc::free_cap_slot(cs);
            free_untyped_4k(ut);
            Err(err)
        }
    }
}

pub fn free_untyped_4k(ut: Untyped) {
    assert!(ut.size_bits() == kernel::PAGE_4K_BITS);
    get_allocator().add_small_page(ut);
}

pub fn free_page4k(page: Page4K) {
    let (ut, cs) = page.free();
    free_untyped_4k(ut);
    crust::capalloc::free_cap_slot(cs);
}