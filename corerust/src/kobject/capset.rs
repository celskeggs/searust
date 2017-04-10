use ::core;
use ::kobject::*;

#[must_use]
pub struct CapSlotSet {
    start: usize,
    end: usize,
    fillstart: usize,
    fillend: usize
}

impl CapSlotSet {
    pub fn empty_range(start: usize, end: usize) -> CapSlotSet {
        let empty_cursor = start; // anywhere in the range will do
        CapSlotSet { start, end, fillstart: empty_cursor, fillend: empty_cursor }
    }

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
            Some(CapSlot::from_index(out))
        } else {
            None
        }
    }

    pub fn take_back(&mut self) -> Option<CapSlot> {
        if self.remaining() {
            self.fillend -= 1;
            Some(CapSlot::from_index(self.fillend))
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
        CapRange::range(self.start, self.end)
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

impl core::ops::Drop for CapSlotSet {
    fn drop(&mut self) {
        if self.remaining() {
            panic!("Cannot drop nonempty CapSlotSets -- this leaks their existence!");
        }
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
