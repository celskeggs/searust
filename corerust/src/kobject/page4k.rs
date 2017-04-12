use ::kobject::*;
use ::core;
use ::mantle;
use ::mantle::KError;
use ::mantle::kernel::{PAGE_4K_SIZE, PAGE_2M_SIZE};
use ::memory::untyped;
use ::memory;
use ::crust;

#[derive(Debug)]
pub struct Page4K {
    cap: Cap,
    parent: Untyped
}

static PAGE_TABLES: mantle::concurrency::SingleThreaded<core::cell::RefCell<memory::LinkedList<MappedPageTable>>> =
    mantle::concurrency::SingleThreaded(core::cell::RefCell::new(memory::LinkedList::empty()));

fn map_page_table(vaddr: usize) -> KError {
    debug!("mapping top-level page table");
    // TODO: make this not break abstraction layers to work properly
    let cslot = match crust::capalloc::allocate_cap_slot() {
        Ok(p) => p,
        Err(err) => {
            debug!("could not map page table");
            return err
        }
    };
    let ut = match untyped::allocate_untyped_4k() {
        Ok(p) => p,
        Err(err) => {
            crust::capalloc::free_cap_slot(cslot);
            debug!("could not allocate untyped");
            return err;
        }
    };
    let page_table: PageTable = match ut.become_page_table(cslot) {
        Ok(p) => p,
        Err((err, ut, cslot)) => {
            untyped::free_untyped_4k(ut);
            crust::capalloc::free_cap_slot(cslot);
            debug!("could not become page");
            return err;
        }
    };
    match page_table.map_into_addr(vaddr) {
        Ok(pt) => {
            assert!(PAGE_TABLES.get().borrow_mut().pushmut(pt).is_ok());
            KError::NoError
        }, Err((pt, err)) => {
            let (ut, cslot) = pt.free();
            untyped::free_untyped_4k(ut);
            crust::capalloc::free_cap_slot(cslot);
            debug!("could not map into address");
            err
        }
    }
}

impl Page4K {
    pub fn from_retyping(cap: Cap, parent: Untyped) -> Page4K {
        Page4K { cap, parent }
    }

    pub fn free(self) -> (Untyped, CapSlot) {
        (self.parent, self.cap.delete())
    }

    fn map_at_address(&self, vaddr: usize, writable: bool) -> KError {
        let crights = if writable { 3 } else { 2 };
        mantle::x86_page_map(self.cap.peek_index(), crust::ROOT_PAGEDIR, vaddr, crights, 0)
    }

    fn unmap(&self) -> KError {
        mantle::x86_page_unmap(self.cap.peek_index())
    }

    pub fn map_into_addr(self, vaddr: usize, writable: bool) -> core::result::Result<FixedMappedPage4K, (Page4K, KError)> {
        let mut err = self.map_at_address(vaddr, writable);
        if err == KError::FailedLookup {
            if map_page_table(vaddr & !(PAGE_2M_SIZE - 1)) == KError::NoError {
                // try again with new page table
                err = self.map_at_address(vaddr, writable);
            }
        }
        if err == KError::NoError {
            Ok(FixedMappedPage4K { page: self, vaddr })
        } else {
            Err((self, err))
        }
    }

    pub fn map_into_vspace(self, writable: bool) -> core::result::Result<RegionMappedPage4K, (Page4K, KError)> {
        match crust::vspace::allocate_vregion(PAGE_4K_SIZE) {
            Ok(vregion) => {
                let mut err = self.map_at_address(vregion.to_4k_address(), writable);
                if err == KError::FailedLookup {
                    if map_page_table(vregion.to_4k_address() & !(PAGE_2M_SIZE - 1)) == KError::NoError {
                        // try again with new page table
                        err = self.map_at_address(vregion.to_4k_address(), writable);
                    }
                }
                if err == KError::NoError {
                    Ok(RegionMappedPage4K { page: self, vregion })
                } else {
                    crust::vspace::free_vregion(vregion);
                    Err((self, err))
                }
            }
            Err(err) => {
                Err((self, err))
            }
        }
    }
}

pub struct FixedMappedPage4K {
    page: Page4K,
    vaddr: usize
}

impl FixedMappedPage4K {
    pub fn get_addr(&self) -> usize {
        self.vaddr
    }

    pub fn get_ptr(&mut self) -> *mut u8 {
        self.get_addr() as *mut u8
    }

    pub fn get_array(&mut self) -> &mut [u8; PAGE_4K_SIZE] {
        let out: &mut [u8; PAGE_4K_SIZE] =
            unsafe { core::mem::transmute((self.get_addr() as *mut [u8; PAGE_4K_SIZE])) };
        out
    }

    pub fn unmap(self) -> Page4K {
        assert!(self.page.unmap() == KError::NoError);
        self.page
    }
}

pub struct RegionMappedPage4K {
    page: Page4K,
    vregion: crust::vspace::VRegion
}

impl RegionMappedPage4K {
    pub fn get_addr(&self) -> usize {
        self.vregion.to_4k_address()
    }

    pub fn get_ptr(&mut self) -> *mut u8 {
        self.get_addr() as *mut u8
    }

    pub fn get_array(&mut self) -> &mut [u8; PAGE_4K_SIZE] {
        let out: &mut [u8; PAGE_4K_SIZE] =
            unsafe { core::mem::transmute((self.get_addr() as *mut [u8; PAGE_4K_SIZE])) };
        out
    }

    pub fn unmap(self) -> Page4K {
        assert!(self.page.unmap() == KError::NoError);
        crust::vspace::free_vregion(self.vregion);
        self.page
    }
}

#[derive(Debug)]
pub struct PageTable {
    cap: Cap,
    parent: Untyped
}

impl PageTable {
    pub fn from_retyping(cap: Cap, parent: Untyped) -> PageTable {
        PageTable { cap, parent }
    }

    pub fn free(self) -> (Untyped, CapSlot) {
        (self.parent, self.cap.delete())
    }

    fn map_at_address(&self, vaddr: usize) -> KError {
        mantle::x86_page_table_map(self.cap.peek_index(), crust::ROOT_PAGEDIR, vaddr, 0)
    }

    fn unmap(&self) -> KError {
        mantle::x86_page_table_unmap(self.cap.peek_index())
    }

    pub fn map_into_addr(self, vaddr: usize) -> core::result::Result<MappedPageTable, (PageTable, KError)> {
        let mut err = self.map_at_address(vaddr);
        if err == KError::NoError {
            Ok(MappedPageTable { page: self })
        } else {
            Err((self, err))
        }
    }
}

pub struct MappedPageTable {
    page: PageTable
}
