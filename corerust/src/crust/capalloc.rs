use ::core;
use ::core::cell::{RefCell, RefMut};
use mantle::concurrency::SingleThreaded;
use mantle::KError;
use kobject::*;
use memory::LinkedList;

static AVAILABLE_CAPS: SingleThreaded<RefCell<LinkedList<CapRange>>> =
    SingleThreaded(RefCell::new(LinkedList::empty()));

fn get_avail_caps_list() -> RefMut<'static, LinkedList<CapRange>> {
    AVAILABLE_CAPS.get().borrow_mut()
}

pub fn allocate_cap_slot() -> core::result::Result<CapSlot, KError> {
    let rl: &mut LinkedList<CapRange> = &mut *get_avail_caps_list();
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
    Ok(CapSlot::from_index(cslot))
}

pub fn allocate_cap_slots(n: usize) -> core::result::Result<CapSlotSet, KError> {
    let rl: &mut LinkedList<CapRange> = &mut *get_avail_caps_list();
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
    let rl: &mut LinkedList<CapRange> = &mut *get_avail_caps_list();
    let mut cur: &mut LinkedList<CapRange> = rl;
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
