use ::kobject::*;
use ::core;
use ::mantle;
use ::mantle::KError;
use ::mantle::kernel::{PAGE_4K_SIZE, PAGE_2M_SIZE};
use ::crust;

#[derive(Debug)]
pub struct Page4K {
    cap: Cap,
    parent: Untyped
}

fn map_page_table(vaddr: usize) -> KError {
    panic!("unimplemented");
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

    pub fn map_into_vspace(self, writable: bool) -> core::result::Result<MappedPage4K, (Page4K, KError)> {
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
                    Ok(MappedPage4K { page: self, vregion })
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

pub struct MappedPage4K {
    page: Page4K,
    vregion: crust::vspace::VRegion
}

impl MappedPage4K {
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
