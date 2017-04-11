use core;
use memory::LinkedList;
use mantle;
use kobject::*;
use mantle::kernel;

pub struct UntypedAllocator {
    small_pages: LinkedList<Untyped>,
    midsize_blocks: LinkedList<Untyped>,
    large_pages: LinkedList<Untyped>,
    oversize_blocks: LinkedList<Untyped>
}

impl UntypedAllocator {
    pub fn mem_available(&self) -> (usize, usize, usize, usize, usize) {
        let (mut oversize_mem, mut large_mem, mut midsize_mem, mut small_mem) = (0usize, 0usize, 0usize, 0usize);
        for block in self.small_pages.into_iter() {
            assert!(block.size_bytes() == kernel::PAGE_4K_SIZE);
            small_mem += block.size_bytes();
        }
        for block in self.midsize_blocks.into_iter() {
            assert!(block.size_bytes() > kernel::PAGE_4K_SIZE);
            assert!(block.size_bytes() < kernel::PAGE_2M_SIZE);
            midsize_mem += block.size_bytes();
        }
        for block in self.large_pages.into_iter() {
            assert!(block.size_bytes() == kernel::PAGE_2M_SIZE);
            large_mem += block.size_bytes();
            debug!("next size: {} {}", block.size_bytes(), large_mem >> 10);
        }
        for block in self.oversize_blocks.into_iter() {
            assert!(block.size_bytes() > kernel::PAGE_2M_SIZE);
            oversize_mem += block.size_bytes();
        }
        (oversize_mem, large_mem, midsize_mem, small_mem, oversize_mem + midsize_mem + large_mem + small_mem)
    }

    pub fn print_info(&self, writer: &mut core::fmt::Write) -> core::fmt::Result {
        let (oversize_mem, large_mem, midsize_mem, small_mem, all_mem) = self.mem_available();
        writeln!(writer, "memory info:")?;
        writeln!(writer, "  number of oversize blocks: {}", self.oversize_blocks.len())?;
        writeln!(writer, "    {} KB", oversize_mem >> 10)?;
        writeln!(writer, "  number of large blocks: {}", self.large_pages.len())?;
        writeln!(writer, "    {} KB", large_mem >> 10)?;
        writeln!(writer, "  number of midsize blocks: {}", self.midsize_blocks.len())?;
        writeln!(writer, "    {} KB", midsize_mem >> 10)?;
        writeln!(writer, "  number of small blocks: {}", self.small_pages.len())?;
        writeln!(writer, "    {} KB", small_mem >> 10)?;
        writeln!(writer, "  total memory: {} KB = {} MB", all_mem >> 10, all_mem >> 20)
    }
}

// ordered from lowest address to highest address
static mut ALLOCATOR: UntypedAllocator =
    UntypedAllocator {
        small_pages: LinkedList::empty(),
        midsize_blocks: LinkedList::empty(),
        large_pages: LinkedList::empty(),
        oversize_blocks: LinkedList::empty()
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
        if ent.is_device == 0 {
            let ll =
                if ent.size_bits > kernel::PAGE_2M_BITS {
                    &mut alloc.oversize_blocks
                } else if ent.size_bits == kernel::PAGE_2M_BITS {
                    &mut alloc.large_pages
                } else if ent.size_bits > kernel::PAGE_4K_BITS {
                    &mut alloc.midsize_blocks
                } else if ent.size_bits == kernel::PAGE_4K_BITS {
                    &mut alloc.small_pages
                } else {
                    panic!("unexpected block is smaller than 4K");
                };
            ll.pushmut(Untyped::from_cap(untyped.nth(i).assert_populated(), ent.size_bits)).unwrap();
        }
    }
    alloc.print_info(mantle::debug());
}
