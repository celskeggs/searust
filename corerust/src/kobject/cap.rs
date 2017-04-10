use ::core;
use ::mantle;
use ::crust;
use ::kobject::*;

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
        core::mem::forget(self);
        out
    }

    pub fn peek_index(&self) -> usize {
        self.index
    }

    pub fn to_range(&self) -> CapRange {
        CapRange::single(self.index)
    }

    pub fn become_set(self) -> CapSlotSet {
        let mut out = self.to_range().to_set_empty();
        out.readd(self);
        out
    }

    pub fn from_index(index: usize) -> CapSlot {
        CapSlot { index }
    }
}

impl core::fmt::Display for CapSlot {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "&{}", self.index)
    }
}

impl core::ops::Drop for CapSlot {
    fn drop(&mut self) {
        panic!("Cannot drop CapSlots -- this leaks their existence!");
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
        assert!(mantle::calls::cnode_delete(crust::ROOT_SLOT, self.peek_index(), crust::ROOT_BITS as u8).is_okay());
        self.loc
    }
}

impl core::fmt::Display for Cap {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "@{}", self.loc.index)
    }
}
