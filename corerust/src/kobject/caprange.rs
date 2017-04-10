use ::kobject::*;
use ::core;

pub struct CapRange {
    // start through end - 1
    start: usize,
    end: usize
}

impl CapRange {
    pub fn single(capslot: usize) -> CapRange {
        CapRange { start: capslot, end: capslot + 1 }
    }

    pub fn range(start: usize, end: usize) -> CapRange {
        CapRange { start, end }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn nth(&self, i: usize) -> CapSlot {
        assert!(i < self.len());
        CapSlot::from_index(self.start + i)
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
        CapSlotSet::empty_range(self.start, self.end)
    }

    pub fn to_set_asserted_full(&self) -> CapSlotSet {
        assert!(self.start < self.end);
        let mut css = self.to_set_empty();
        css.assert_full();
        css
    }
}

impl core::fmt::Display for CapRange {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "[{}, {})", self.start, self.end)
    }
}
