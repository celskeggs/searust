use ::core::*;
use ::kobj;
use ::sel4::KError;
use ::caps::*;

#[repr(usize)]
pub enum ObjectType {
    seL4_UntypedObject = 0,
    seL4_TCBObject = 1,
    seL4_EndpointObject = 2,
    seL4_NotificationObject = 3,
    seL4_CapTableObject = 4,
    seL4_X86_PDPTObject = 5,
    seL4_X64_PML4Object = 6,
    seL4_X86_4K = 7,
    seL4_X86_LargePageObject = 8,
    seL4_X86_PageTableObject = 9,
    seL4_X86_PageDirectoryObject = 10,
}

pub const PAGE_4K_BITS: u8 = 12;

#[derive(Debug)]
pub struct Untyped {
    cap: Cap,
    size_bits: u8
}

impl Untyped {
    pub fn from_cap(cap: Cap, size_bits: u8) -> Untyped {
        Untyped { cap, size_bits }
    }

    pub fn size_bits(&self) -> u8 {
        self.size_bits
    }

    fn retype_raw(&self, objtype: ObjectType, size_bits: u8, mut capslots: CapSlotSet)
                  -> result::Result<CapSet, (KError, CapSlotSet)> {
        assert!(capslots.capacity() > 0);
        assert!(capslots.full());
        assert!(capslots.count() > 0);
        assert!(capslots.count() == capslots.capacity());
        let err = kobj::sel4_untyped_retype(self.cap.peek_index(), objtype as usize, size_bits as usize,
                                            ::caps::ROOT_SLOT, 0, 0,
                                            capslots.start(), capslots.count());
        if err.is_okay() {
            Ok(capslots.assert_derive_capset())
        } else {
            // TODO: are we sure there isn't something more complicated that happens on failure? like partial completion?
            Err((err, capslots))
        }
    }

    fn retype_raw_one(&self, objtype: ObjectType, size_bits: u8, capslot: CapSlot) -> result::Result<Cap, (KError, CapSlot)> {
        match self.retype_raw(objtype, size_bits, capslot.become_set()) {
            Ok(mut capset) => Ok(capset.take_front().unwrap()),
            Err((err, mut capslotset)) => Err((err, capslotset.take_front().unwrap()))
        }
    }

    pub fn split(self, split_bits: u8, capslots: CapSlotSet) -> result::Result<UntypedSet, (KError, Untyped, CapSlotSet)> {
        assert!(capslots.full());
        assert!((1 << split_bits) == capslots.capacity());
        let final_size_bits = self.size_bits - split_bits;
        assert!(final_size_bits >= 4);
        match self.retype_raw(ObjectType::seL4_UntypedObject, final_size_bits, capslots) {
            Ok(capset) => Ok(UntypedSet { capset, size_bits: final_size_bits, parent: self }),
            Err((err, capslotset)) => Err((err, self, capslotset))
        }
    }

    pub fn become_page_4k(self, capslot: CapSlot) -> result::Result<Page4K, (KError, Untyped, CapSlot)> {
        assert!(self.size_bits == PAGE_4K_BITS);
        match self.retype_raw_one(ObjectType::seL4_X86_4K, 0, capslot) {
            Ok(cap) => Ok(Page4K { cap, parent: self }),
            Err((err, capslot)) => Err((err, self, capslot))
        }
    }
}

impl fmt::Display for Untyped {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "untyped {}-bit in {}", self.size_bits, &self.cap)
    }
}

pub struct UntypedSet {
    capset: CapSet,
    size_bits: u8,
    parent: Untyped
}

impl UntypedSet {
    pub fn free(self) -> (Untyped, CapSlotSet) {
        assert!(self.capset.full());
        (self.parent, self.capset.delete_all())
    }

    pub fn capacity(&self) -> usize {
        self.capset.capacity()
    }

    pub fn count(&self) -> usize {
        self.capset.count()
    }

    pub fn remaining(&self) -> bool {
        self.capset.remaining()
    }

    pub fn full(&self) -> bool {
        self.capset.full()
    }

    pub fn take_front(&mut self) -> Option<Untyped> {
        if let Some(cap) = self.capset.take_front() {
            Some(Untyped { cap, size_bits: self.size_bits })
        } else {
            None
        }
    }

    pub fn take_back(&mut self) -> Option<Untyped> {
        if let Some(cap) = self.capset.take_back() {
            Some(Untyped { cap, size_bits: self.size_bits })
        } else {
            None
        }
    }

    pub fn readd(&mut self, slot: Untyped) {
        assert!(slot.size_bits == self.size_bits);
        self.capset.readd(slot.cap);
    }
}

impl fmt::Display for UntypedSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "untypedset {}-bit with {}/{} left", self.size_bits, self.capset.remaining(), self.capset.count())
    }
}

#[derive(Debug)]
pub struct Page4K {
    cap: Cap,
    parent: Untyped
}

impl Page4K {
    pub fn free(self) -> (Untyped, CapSlot) {
        (self.parent, self.cap.delete())
    }
}
