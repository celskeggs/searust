use ::core;
use ::crust;
use ::mantle::KError;
use ::kobject::*;
use ::memory;

struct FragmentAllocator {
    available_fragments: memory::LinkedList<Untyped>,
    expired_sets: memory::LinkedList<UntypedSet>
}

impl FragmentAllocator {
    fn refill(&mut self) -> core::result::Result<(), KError> {
        assert!(self.available_fragments.is_empty());
        let large_ut = memory::untyped::allocate_untyped_4k()?;
        // we want to go from 12 bits to 4 bits: 8 bit difference
        match large_ut.split_calloc(8) {
            Ok(mut untypedset) => {
                while let Some(ent) = untypedset.take_front() {
                    assert!(ent.size_bits() == 4);
                    self.available_fragments.pushmut(ent);
                }
                self.expired_sets.pushmut(untypedset);
                Ok(())
            }, Err((err, large_ut)) => {
                memory::untyped::free_untyped_4k(large_ut);
                Err(err)
            }
        }
    }

    fn allocate(&mut self) -> core::result::Result<Untyped, KError> {
        if self.available_fragments.is_empty() {
            self.refill()?;
        }
        self.available_fragments.popmut().ok_or(KError::NotEnoughMemory)
    }

    fn free(&mut self, ut: Untyped) {
        assert!(ut.size_bits() == 4);
        self.available_fragments.pushmut(ut);
    }
}

static mut FRAGMENT_ALLOC: FragmentAllocator = FragmentAllocator { available_fragments: memory::LinkedList::empty(), expired_sets: memory::LinkedList::empty() };

pub fn allocate_untyped_16b() -> core::result::Result<Untyped, KError> {
    unsafe {
        &mut FRAGMENT_ALLOC
    }.allocate()
}

pub fn free_untyped_16b(ut: Untyped) {
    unsafe {
        &mut FRAGMENT_ALLOC
    }.free(ut)
}

pub fn allocate_notification() -> core::result::Result<Notification, KError> {
    let ut: Untyped = allocate_untyped_16b()?;
    match crust::capalloc::allocate_cap_slot() {
        Ok(slot) => {
            match ut.become_notification(slot) {
                Ok(noti) => Ok(noti),
                Err((err, ut, slot)) => {
                    crust::capalloc::free_cap_slot(slot);
                    free_untyped_16b(ut);
                    Err(err)
                }
            }
        },
        Err(err) => {
            free_untyped_16b(ut);
            Err(err)
        }
    }
}

pub fn free_notification(not: Notification) {
    let (ut, slot) = not.free();
    crust::capalloc::free_cap_slot(slot);
    free_untyped_16b(ut)
}