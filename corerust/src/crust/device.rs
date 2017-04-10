use memory::LinkedList;
use crust::capalloc;
use core;
use mantle::kernel;
use mantle::KError;
use kobject::*;
use core::cell::{RefCell, RefMut};
use core::ops::DerefMut;

struct Subblock {
    ut: Option<Untyped>,
    us: Option<UntypedSet>,
    paddr: usize,
    size_bits: u8
}

impl Subblock {
    pub fn start(&self) -> usize {
        self.paddr
    }

    pub fn len(&self) -> usize {
        1 << (self.size_bits as usize)
    }

    pub fn mid(&self) -> usize {
        self.paddr + (1 << ((self.size_bits - 1) as usize))
    }

    pub fn end(&self) -> usize {
        self.paddr + self.len()
    }

    pub fn contains(&self, addr: usize) -> bool {
        self.start() <= addr && addr < self.end()
    }

    pub fn is_available(&self) -> bool {
        self.ut.is_some()
    }

    pub fn take(&mut self) -> Untyped {
        core::mem::replace(&mut self.ut, None).unwrap()
    }

    pub fn return_taken(&mut self, ut: Untyped) {
        assert!(self.size_bits == ut.size_bits());
        assert!(self.ut.is_none());
        assert!(self.us.is_none());
        self.ut = Some(ut);
    }

    pub fn split(&mut self) -> core::result::Result<(Subblock, Subblock), KError> {
        let ut = self.take();
        match ut.split(1, capalloc::allocate_cap_slots(2)?) {
            Ok(mut uset) => {
                assert!(self.us.is_none());
                let earlier = uset.take_front().unwrap();
                let later = uset.take_front().unwrap();
                self.us = Some(uset);
                Ok((Subblock { ut: Some(earlier), us: None, paddr: self.start(), size_bits: self.size_bits - 1 },
                    Subblock { ut: Some(later), us: None, paddr: self.mid(), size_bits: self.size_bits - 1 }))
            }
            Err((err, ut, slots)) => {
                assert!(self.ut.is_none());
                self.ut = Some(ut);
                capalloc::free_cap_slots(slots);
                Err(err)
            }
        }
    }

    pub fn unsplit(&mut self, earlierblk: Subblock, laterblk: Subblock) {
        let mut uset = core::mem::replace(&mut self.us, None).unwrap();
        uset.readd(laterblk.ut.unwrap());
        uset.readd(earlierblk.ut.unwrap());
        let (untyped, capslotset) = uset.free();
        capalloc::free_cap_slots(capslotset);
        self.return_taken(untyped);
    }

    pub fn needs_split(&self) -> bool {
        assert!(self.size_bits >= kernel::PAGE_4K_BITS);
        self.size_bits != kernel::PAGE_4K_BITS
    }

    pub fn try_use_as_page(&mut self) -> core::result::Result<core::result::Result<Untyped, (Subblock, Subblock)>, KError> {
        if self.needs_split() {
            Ok(Err(self.split()?))
        } else {
            Ok(Ok(self.take()))
        }
    }
}

impl core::fmt::Display for Subblock {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if let Some(ref ut) = self.ut {
            assert!(self.us.is_none());
            write!(f, "Subblock {} @ {:#X}-{:#X}", ut, self.start(), self.end())
        } else if let Some(ref us) = self.us {
            write!(f, "Subblock split {} @ {:#X}-{:#X}", us, self.start(), self.end())
        } else {
            write!(f, "Subblock taken {}-bit {:#X}-{:#X}", self.size_bits, self.start(), self.end())
        }
    }
}

pub struct DeviceBlock {
    caps: LinkedList<RefCell<Subblock>>,
    // invariant: subblocks always come BEFORE superblocks
    size_bits: u8,
    paddr: usize
}

impl DeviceBlock {
    pub fn new(ut: Untyped, paddr: usize) -> DeviceBlock {
        let bits = ut.size_bits();
        match LinkedList::empty().push(RefCell::new(Subblock { ut: Some(ut), us: None, paddr, size_bits: bits })) {
            Ok(ll) => DeviceBlock { caps: ll, size_bits: bits, paddr },
            Err(_) => panic!("could not allocate memory for DeviceBlock")
        }
    }

    pub fn start(&self) -> usize {
        self.paddr
    }

    pub fn len(&self) -> usize {
        1 << (self.size_bits as usize)
    }

    pub fn end(&self) -> usize {
        self.paddr + self.len()
    }

    pub fn contains(&self, addr: usize) -> bool {
        self.start() <= addr && addr < self.end()
    }

    fn device_scan<'a>(&'a self, ll: &'a LinkedList<RefCell<Subblock>>, addr: usize) -> core::result::Result<usize, KError> {
        assert!(self.caps.is_empty()); // not currently valid
        assert!(self.contains(addr));
        if let Some((index, found)) = ll.find_and_index(|b| b.borrow().contains(addr)) {
            if found.borrow().is_available() {
                Ok(index)
            } else {
                debug!("failed lookup of {} due to lack of availability", addr);
                Err(KError::FailedLookup)
            }
        } else {
            // should be guaranteed to find it, since everything is just a subblock of the overall
            // thing, and that containment should already be checked.
            panic!("could not look up expected address {:#X}", addr);
        }
    }

    /**
      * This takes an earlier-in-memory subblock and a later-in-memory subblock, adds them to the
      * list, and returns a reference to the one that includes paddr.
      */
    // the implicit RefMut is the first element of the linked list
    fn device_split_iter<'a>(ll: LinkedList<RefCell<Subblock>>, earlier: Subblock, later: Subblock, paddr: usize) -> core::result::Result<LinkedList<RefCell<Subblock>>, (Subblock, Subblock, LinkedList<RefCell<Subblock>>)> {
        let later_has_paddr: bool = later.contains(paddr);
        assert!(earlier.contains(paddr) != later_has_paddr);
        if later_has_paddr {
            match ll.push(RefCell::new(earlier)) {
                Ok(ll) => {
                    match ll.push(RefCell::new(later)) {
                        Ok(ll) => Ok(ll),
                        Err((ll, later)) => {
                            let (earlier, ll2) = ll.pop().unwrap();
                            Err((earlier.into_inner(), later.into_inner(), ll2))
                        }
                    }
                }
                Err((ll, earlier)) => {
                    Err((earlier.into_inner(), later, ll))
                }
            }
        } else {
            match ll.push(RefCell::new(later)) {
                Ok(ll) => {
                    match ll.push(RefCell::new(earlier)) {
                        Ok(ll) => Ok(ll),
                        Err((ll, earlier)) => {
                            let (later, ll2) = ll.pop().unwrap();
                            Err((earlier.into_inner(), later.into_inner(), ll2))
                        }
                    }
                }
                Err((ll, later)) => {
                    Err((earlier, later.into_inner(), ll))
                }
            }
        }
    }

    fn iter_i(i: usize, ll: LinkedList<RefCell<Subblock>>, addr: usize) -> core::result::Result<core::result::Result<(Untyped, LinkedList<RefCell<Subblock>>), LinkedList<RefCell<Subblock>>>, (KError, LinkedList<RefCell<Subblock>>)> {
        let page = ll.get(i).unwrap().borrow_mut().deref_mut().try_use_as_page();
        if let Err(err) = page {
            return Err((page.err().unwrap(), ll));
        }
        let rest = page.ok().unwrap();
        if let Ok(ut) = rest {
            return Ok(Ok((ut, ll)));
        }
        let (earlier, later) = rest.err().unwrap();
        match DeviceBlock::device_split_iter(ll, earlier, later, addr) {
            Ok(ncur) => {
                Ok(Err(ncur))
            }
            Err((earlier, later, ll)) => {
                ll.get(i).unwrap().borrow_mut().deref_mut().unsplit(earlier, later);
                Err((KError::NotEnoughMemory, ll))
            }
        }
    }

    fn get_device_page_untyped_ll(&mut self, ll: LinkedList<RefCell<Subblock>>, addr: usize) -> (core::result::Result<Untyped, KError>, LinkedList<RefCell<Subblock>>) {
        assert!(self.caps.is_empty()); // not currently valid
        assert!(self.size_bits >= kernel::PAGE_4K_BITS);
        // be very careful with try! here.
        let mut ri = match self.device_scan(&ll, addr) {
            Ok(ri) => ri,
            Err(err) => {
                return (Err(err), ll);
            }
        };
        let mut cur: LinkedList<RefCell<Subblock>> = ll;
        loop {
            match DeviceBlock::iter_i(ri, cur, addr) {
                Ok(Ok((ut, ll2))) => {
                    return (Ok(ut), ll2);
                }
                Ok(Err(iter)) => {
                    cur = iter;
                    ri = 0;
                }
                Err((err, ll2)) => {
                    return (Err(err), ll2);
                }
            };
        }
    }

    pub fn get_device_page_untyped(&mut self, addr: usize) -> core::result::Result<Untyped, KError> {
        assert!(self.contains(addr));
        let ll = core::mem::replace(&mut self.caps, LinkedList::Empty);
        let (res, ll) = self.get_device_page_untyped_ll(ll, addr);
        self.caps = ll;
        res
    }

    pub fn return_device_page_untyped(&mut self, addr: usize, untyped: Untyped) {
        // TODO: check this logic... I'm not sure things are being returned to the exact correct places...
        assert!(self.size_bits >= kernel::PAGE_4K_BITS);
        assert!(untyped.size_bits() == kernel::PAGE_4K_BITS);
        assert!(self.contains(addr));
        let mut foundref: RefMut<Subblock> = self.caps.find(|b| b.borrow().contains(addr)).unwrap().borrow_mut();
        let found: &mut Subblock = foundref.deref_mut();
        assert!(!found.is_available());
        assert!(found.size_bits == kernel::PAGE_4K_BITS);
        found.return_taken(untyped);
    }

    pub fn get_device_page(&mut self, addr: usize, slot: CapSlot) -> core::result::Result<Page4K, (KError, CapSlot)> {
        match self.get_device_page_untyped(addr) {
            Ok(untyped) => {
                match untyped.become_page_4k(slot) {
                    Ok(page) => Ok(page),
                    Err((err, untyped, slot)) => {
                        self.return_device_page_untyped(addr, untyped);
                        Err((err, slot))
                    }
                }
            }
            Err(err) => {
                Err((err, slot))
            }
        }
    }

    pub fn return_device_page(&mut self, addr: usize, page: Page4K) -> CapSlot {
        let (ut, cs) = page.free();
        self.return_device_page_untyped(addr, ut);
        cs
    }
}

impl ::core::fmt::Display for DeviceBlock {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        if self.caps.is_empty() {
            write!(f, "[malformed")?;
        } else {
            write!(f, "[{}", *self.caps.head().unwrap().borrow())?;
            for iter in self.caps.tail().unwrap() {
                let elem: RefMut<Subblock> = iter.borrow_mut();
                write!(f, ", {}", *elem)?;
            }
        }
        write!(f, "] => {:#X}-{:#X}", self.start(), self.end())
    }
}

impl ::core::fmt::Debug for DeviceBlock {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        if self.caps.is_empty() {
            write!(f, "[malformed")?;
        } else {
            write!(f, "DeviceBlock([{}", *self.caps.head().unwrap().borrow())?;
            for iter in self.caps.tail().unwrap() {
                let elem: RefMut<Subblock> = iter.borrow_mut();
                write!(f, ", {}", *elem)?;
            }
        }
        write!(f, " => {:#X}-{:#X})", self.start(), self.end())
    }
}

// ordered from lowest address to highest address
static mut DEVICES: Option<::memory::LinkedList<DeviceBlock>> = None;

pub fn get_device_list() -> &'static mut ::memory::LinkedList<DeviceBlock> {
    let dev: &mut Option<::memory::LinkedList<DeviceBlock>> = unsafe { &mut DEVICES };
    if let &mut Some(ref mut out) = dev {
        out
    } else {
        panic!("device listing not yet initialized!");
    }
}

pub fn get_containing_block(addr: usize) -> Option<&'static mut DeviceBlock> {
    get_device_list().find_mut(|dev| dev.contains(addr))
}

pub fn get_device_page(addr: usize) -> core::result::Result<Page4K, KError> {
    if let Some(block) = get_containing_block(addr) {
        let slot = capalloc::allocate_cap_slot()?;
        match block.get_device_page(addr, slot) {
            Ok(page) => Ok(page),
            Err((err, slot)) => {
                capalloc::free_cap_slot(slot);
                Err(err)
            }
        }
    } else {
        debug!("failed to lookup {:#X} due to block unavailable", addr);
        Err(KError::FailedLookup)
    }
}

pub fn return_device_page(addr: usize, page: Page4K) {
    if let Some(block) = get_containing_block(addr) {
        let slot = block.return_device_page(addr, page);
        capalloc::free_cap_slot(slot);
    } else {
        panic!("attempt to return device page to block that never existed");
    }
}

pub fn get_mapped_device_page(addr: usize) -> core::result::Result<MappedPage4K, KError> {
    let page = get_device_page(addr)?;
    match page.map_into_vspace(true) {
        Ok(mapping) => {
            Ok(mapping)
        }
        Err((page, err)) => {
            return_device_page(addr, page);
            Err(err)
        }
    }
}

pub fn return_mapped_device_page(addr: usize, page: MappedPage4K) {
    return_device_page(addr, page.unmap());
}

pub fn init_untyped(untyped: CapRange, untyped_list: [kernel::UntypedDesc; 230usize]) {
    let count = untyped.len();
    // these are sorted!
    let mut devices = ::memory::LinkedList::empty();
    let mut last_addr: usize = (-1 as isize) as usize;
    for ir in 0..count {
        let i = count - 1 - ir;
        let ent = untyped_list[i];
        if ent.is_device != 0 {
            let newblock = DeviceBlock::new(Untyped::from_cap(untyped.nth(i).assert_populated(), ent.size_bits), ent.paddr as usize);
            assert!(newblock.end() <= last_addr);
            last_addr = newblock.start();
            devices = match devices.push(newblock) {
                Ok(devs) => devs,
                Err(_) => panic!("could not allocate memory for device list")
            };
        }
    }
    for dev in &devices {
        debug!("dev {} of {} bits", dev, dev.size_bits);
    }
    unsafe {
        assert!(DEVICES.is_none());
        DEVICES = Some(devices);
    }
}
