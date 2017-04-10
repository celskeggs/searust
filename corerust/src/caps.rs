use ::kobj;
use ::concurrency::SingleThreaded;
use ::core::cell::RefCell;
use ::core::cell::RefMut;
use ::sel4::KError;
use ::core;
use ::libsel4;
use ::memory;

#[must_use]
#[derive(Debug)]
pub struct CapSlot {
    index: usize
}

impl CapSlot {
    pub fn assert_populated(self) -> Cap {
        Cap { loc: self }
    }

    pub fn deconstruct(self) -> usize {
        let out = self.index;
        ::core::mem::forget(self);
        out
    }

    pub fn peek_index(&self) -> usize {
        self.index
    }

    pub fn to_range(&self) -> CapRange {
        CapRange { start: self.index, end: self.index + 1 }
    }

    pub fn become_set(self) -> CapSlotSet {
        let mut out = self.to_range().to_set_empty();
        out.readd(self);
        out
    }
}

impl ::core::fmt::Display for CapSlot {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "&{}", self.index)
    }
}

impl ::core::ops::Drop for CapSlot {
    fn drop(&mut self) {
        panic!("Cannot drop CapSlots -- this leaks their existence!");
    }
}

#[must_use]
pub struct CapSlotSet {
    start: usize,
    end: usize,
    fillstart: usize,
    fillend: usize
}

impl CapSlotSet {
    pub fn start(&self) -> usize {
        self.start
    }
    pub fn capacity(&self) -> usize {
        self.end - self.start
    }

    pub fn count(&self) -> usize {
        self.fillend - self.fillstart
    }

    pub fn remaining(&self) -> bool {
        assert!(self.fillend >= self.fillstart);
        self.fillstart != self.fillend
    }

    pub fn full(&self) -> bool {
        assert!(self.fillend >= self.fillstart);
        assert!(self.fillstart >= self.start && self.fillend <= self.end);
        self.fillstart == self.start && self.fillend == self.end
    }

    pub fn assert_full(&mut self) {
        assert!(self.start < self.end);
        self.fillstart = self.start;
        self.fillend = self.end;
    }

    pub fn assert_empty(&mut self) {
        assert!(self.start < self.end);
        self.fillstart = self.start;
        self.fillend = self.start;
    }

    pub fn take_front(&mut self) -> Option<CapSlot> {
        if self.remaining() {
            let out = self.fillstart;
            self.fillstart += 1;
            Some(CapSlot { index: out })
        } else {
            None
        }
    }

    pub fn take_back(&mut self) -> Option<CapSlot> {
        if self.remaining() {
            self.fillend -= 1;
            Some(CapSlot { index: self.fillend })
        } else {
            None
        }
    }

    pub fn readd(&mut self, slot: CapSlot) {
        let index = slot.deconstruct();
        if self.remaining() {
            // must be contiguous
            if self.fillstart == index + 1 {
                assert!(self.fillstart > self.start);
                self.fillstart -= 1
            } else if self.fillend == index {
                assert!(self.fillend < self.end);
                self.fillend += 1;
            }
        } else {
            // can be anywhere
            assert!(self.start <= index && index < self.end);
            self.fillstart = index;
            self.fillend = index + 1;
        }
    }

    pub fn deconstruct(mut self) -> CapRange {
        assert!(self.full());
        self.assert_empty();
        self.equivalent_range()
    }

    pub fn equivalent_range(&self) -> CapRange {
        CapRange { start: self.start, end: self.end }
    }

    pub fn equivalent_empty_slotset(&self) -> CapSlotSet {
        self.equivalent_range().to_set_empty()
    }

    pub fn equivalent_empty_set(&self) -> CapSet {
        CapSet { backing: self.equivalent_empty_slotset() }
    }

    pub fn assert_derive_capset(&mut self) -> CapSet {
        let mut out: CapSet = self.equivalent_empty_set();
        out.assert_full();
        self.assert_empty();
        out
    }
}

impl ::core::ops::Drop for CapSlotSet {
    fn drop(&mut self) {
        if self.remaining() {
            panic!("Cannot drop nonempty CapSlotSets -- this leaks their existence!");
        }
    }
}

pub struct CapRange {
    // start through end - 1
    start: usize,
    end: usize
}

impl CapRange {
    pub fn single(capslot: usize) -> CapRange {
        CapRange { start: capslot, end: capslot + 1 }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn nth(&self, i: usize) -> CapSlot {
        assert!(i < self.len());
        CapSlot { index: self.start + i }
    }

    pub fn is_empty(&self) -> bool {
        assert!(self.end >= self.start);
        self.end == self.start
    }

    pub fn is_after(&self, other: &CapRange) -> bool {
        self.start >= other.start
    }

    pub fn chop_1(&mut self) -> Option<usize> {
        if self.is_empty() {
            None
        } else {
            self.start += 1;
            Some(self.start - 1)
        }
    }

    pub fn chop_n(&mut self, n: usize) -> Option<CapRange> {
        if n > self.len() {
            None
        } else {
            let out = CapRange { start: self.start, end: self.start + n };
            self.start += n;
            assert!(self.end >= self.start);
            Some(out)
        }
    }

    pub fn intersection(&self, other: &CapRange) -> Option<CapRange> {
        let (lower, higher) = if self.start < other.start {
            (self, other)
        } else {
            (other, self)
        };
        if lower.end > higher.start {
            Some(CapRange { start: higher.start, end: lower.end })
        } else {
            None
        }
    }

    pub fn join(self, other: CapRange) -> core::result::Result<CapRange, (CapRange, CapRange)> {
        assert!(self.intersection(&other).is_none()); // intersections are BAD
        if self.end == other.start {
            Ok(CapRange { start: self.start, end: other.end })
        } else if self.start == other.end {
            Ok(CapRange { start: other.start, end: self.end })
        } else {
            Err((self, other))
        }
    }

    pub fn join_mut(&mut self, other: CapRange) -> Option<CapRange> {
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

    pub fn could_join(&self, other: &CapRange) -> bool {
        assert!(self.intersection(&other).is_none()); // intersections are BAD
        self.end == other.start || self.start == other.end
    }

    pub fn to_set_empty(&self) -> CapSlotSet {
        assert!(self.start < self.end);
        let empty_cursor = self.start; // any place in the range works
        CapSlotSet { start: self.start, end: self.end, fillstart: empty_cursor, fillend: empty_cursor }
    }

    pub fn to_set_asserted_full(&self) -> CapSlotSet {
        assert!(self.start < self.end);
        CapSlotSet { start: self.start, end: self.end, fillstart: self.start, fillend: self.end }
    }
}

impl CapRange {
    pub fn range(start: usize, end: usize) -> CapRange {
        CapRange { start, end }
    }
}

impl ::core::fmt::Display for CapRange {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "[{}, {})", self.start, self.end)
    }
}

#[must_use]
#[derive(Debug)]
pub struct Cap {
    loc: CapSlot
}

impl Cap {
    pub fn assert_unpopulated(self) -> CapSlot {
        self.loc
    }

    pub fn peek_slot(&self) -> &CapSlot {
        &self.loc
    }

    pub fn peek_index(&self) -> usize {
        self.loc.peek_index()
    }

    pub fn delete(self) -> CapSlot {
        let err = kobj::sel4_cnode_delete(ROOT_SLOT, self.peek_index(), ROOT_BITS as u8);
        assert!(err.is_okay());
        self.loc
    }
}

impl ::core::fmt::Display for Cap {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "@{}", self.loc.index)
    }
}

#[must_use]
pub struct CapSet {
    backing: CapSlotSet
}

impl CapSet {
    pub fn start(&self) -> usize {
        self.backing.start()
    }

    pub fn capacity(&self) -> usize {
        self.backing.capacity()
    }

    pub fn count(&self) -> usize {
        self.backing.count()
    }

    pub fn remaining(&self) -> bool {
        self.backing.remaining()
    }

    pub fn full(&self) -> bool {
        self.backing.full()
    }

    pub fn assert_full(&mut self) {
        self.backing.assert_full()
    }

    pub fn assert_empty(&mut self) {
        self.backing.assert_empty()
    }

    pub fn delete_all(mut self) -> CapSlotSet {
        let mut slotset = self.equivalent_empty_slotset();
        while let Some(cap) = self.take_front() {
            slotset.readd(cap.delete());
        }
        slotset
    }

    pub fn take_front(&mut self) -> Option<Cap> {
        if let Some(slot) = self.backing.take_front() {
            Some(slot.assert_populated())
        } else {
            None
        }
    }

    pub fn take_back(&mut self) -> Option<Cap> {
        if let Some(slot) = self.backing.take_back() {
            Some(slot.assert_populated())
        } else {
            None
        }
    }

    pub fn readd(&mut self, slot: Cap) {
        self.backing.readd(slot.assert_unpopulated())
    }

    pub fn equivalent_range(&self) -> CapRange {
        self.backing.equivalent_range()
    }

    pub fn equivalent_empty_slotset(&self) -> CapSlotSet {
        self.backing.equivalent_empty_slotset()
    }

    pub fn equivalent_empty_set(&self) -> CapSet {
        self.backing.equivalent_empty_set()
    }
}

pub const ROOT_SLOT: usize = libsel4::seL4_CapInitThreadCNode as u32 as usize;
pub const ROOT_PAGEDIR: usize = libsel4::seL4_CapInitThreadVSpace as u32 as usize;
pub const ROOT_BITS: usize = 64; // TODO: maybe this should be 32?

static AVAILABLE_CAPS: SingleThreaded<RefCell<memory::LinkedList<CapRange>>> = SingleThreaded(RefCell::new(memory::LinkedList::empty()));

fn get_avail_caps_list() -> RefMut<'static, memory::LinkedList<CapRange>> {
    AVAILABLE_CAPS.get().borrow_mut()
}

pub fn allocate_cap_slot() -> core::result::Result<CapSlot, KError> {
    let rl: &mut memory::LinkedList<CapRange> = &mut *get_avail_caps_list();
    let (cslot, is_now_empty): (usize, bool) = {
        let h = rl.headmut();
        if h.is_none() {
            return Err(KError::NotEnoughMemory);
        }
        let head = h.unwrap();
        (head.chop_1().unwrap(), head.is_empty())
    };
    if is_now_empty {
        assert!(rl.popmut().unwrap().is_empty());
    }
    debug!("allocated slot {}", cslot);
    Ok(CapSlot { index: cslot })
}

pub fn allocate_cap_slots(n: usize) -> core::result::Result<CapSlotSet, KError> {
    let rl: &mut memory::LinkedList<CapRange> = &mut *get_avail_caps_list();
    let (crange, is_now_empty): (CapRange, bool) = {
        let h = rl.find_mut(|b| b.len() >= n);
        if h.is_none() {
            return Err(KError::NotEnoughMemory);
        }
        let head = h.unwrap();
        (head.chop_n(n).unwrap(), head.is_empty())
    };

    if is_now_empty {
        assert!(rl.remove_mut(|b| b.is_empty()).unwrap().is_empty());
        assert!(rl.find(|b| b.is_empty()).is_none());
    }
    debug!("allocated slot range {}", crange);
    Ok(crange.to_set_asserted_full())
}

fn merge_caprange(mut r: CapRange) {
    assert!(!r.is_empty());
    let rl: &mut memory::LinkedList<CapRange> = &mut *get_avail_caps_list();
    let mut cur: &mut memory::LinkedList<CapRange> = rl;
    loop {
        let tmp = cur;
        if tmp.is_empty() {
            // nope -- cur is the end of the line! just add our stuff.
            if tmp.pushmut(r).is_err() {
                panic!("could not free caprange due to OOM condition");
            }
            // added!
            return;
        }
        let (head, ncur) = tmp.nextmut().unwrap();
        // try to merge this one
        if let Some(rest) = head.join_mut(r) {
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
                assert!(head.join_mut(ncur.popmut().unwrap()).is_none()); // make sure we're successful
            }
            return;
        }
        cur = ncur
    }
}

pub fn free_cap_slot(cs: CapSlot) {
    merge_caprange(CapRange::single(cs.deconstruct()));
}

pub fn free_cap_slots(cs: CapSlotSet) {
    assert!(cs.capacity() > 0);
    merge_caprange(cs.deconstruct());
}

pub fn init_cslots(cs: CapRange) {
    assert!(!cs.is_empty());
    merge_caprange(cs);
}