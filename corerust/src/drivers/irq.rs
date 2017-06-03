use ::core;
use ::crust;
use ::memory;
use ::kobject::*;
use ::mantle;
use ::mantle::concurrency::SingleThreaded;
use ::mantle::KError;
use ::core::cell::RefCell;
use ::core::cell::RefMut;
use ::memory::Box;

pub const IRQ_MAX: u32 = 32;

struct IRQManager {
    irqcontrol: IRQControl,
    notification: Notification,
    callbacks: RefCell<[Option<Box<FnMut()>>; IRQ_MAX as usize]>
}

pub struct IRQ<'a> {
    irqhandler: IRQHandler,
    manager: &'a IRQManager,
    irq: u32
}

impl IRQManager {
    fn new() -> core::result::Result<IRQManager, KError> {
        let notify = memory::smalluntyped::allocate_notification()?;
        let irqc = IRQControl::from_cap(CapSlot::from_index(mantle::kernel::CAP_INIT_IRQCONTROL).assert_populated());
        Ok(IRQManager { irqcontrol: irqc, notification: notify, callbacks: RefCell::new(
            [None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None]) })
    }

    fn request<'a>(&'a self, irq: u32) -> core::result::Result<IRQ<'a>, KError> {
        assert!(irq < IRQ_MAX); // make sure it fits in a notification word
        let cslot = crust::capalloc::allocate_cap_slot()?;
        match self.irqcontrol.get(irq, cslot) {
            Ok(irqhandler) => {
                irqhandler.set_notification(&self.notification);
                Ok(IRQ { irqhandler, manager: self, irq })
            }
            Err((err, cslot)) => {
                crust::capalloc::free_cap_slot(cslot);
                Err(err)
            }
        }
    }

    fn on_bit(&mut self, bit: u32) {
        if let &Some(ref cb) = &self.callbacks.borrow()[bit as usize] {
            debug!("invoking IRQ callback for {}", bit);
            cb();
        } else {
            debug!("no IRQ callback registered for {}", bit);
        }
    }

    fn mainloop(&mut self) {
        loop {
            let mut sender = self.notification.wait();
            debug!("got IRQ notification: {:#b}", sender);
            while sender != 0 {
                let active_bit = sender.trailing_zeros();
                self.on_bit(active_bit);
                sender &= !(1 << active_bit);
                assert!(sender == 0 || sender.trailing_zeros() > active_bit);
            }
        }
    }

    fn set_callback<F: Fn() + 'static>(&self, irq: u32, cb: F) {
        assert!(self.callbacks.borrow()[irq as usize].is_none());
        self.callbacks.borrow_mut()[irq as usize] = Some(Box::new(cb));
    }

    fn clear_callback(&self, irq: u32) {
        assert!(self.callbacks.borrow()[irq as usize].is_some());
        self.callbacks.borrow_mut()[irq as usize] = None;
    }
}

impl<'a> IRQ<'a> {
    pub fn free(self) {
        self.clear_cb();
        assert!(self.irqhandler.clear().is_ok());
        crust::capalloc::free_cap_slot(self.irqhandler.free())
    }

    pub fn ack(&self) -> core::result::Result<(), KError> {
        self.irqhandler.ack()
    }

    pub fn set_cb<F: Fn() + 'static>(&self, cb: F) {
        self.manager.set_callback(self.irq, cb)
    }

    pub fn clear_cb(&self) {
        self.manager.clear_callback(self.irq);
    }
}

static MANAGER: SingleThreaded<RefCell<Option<IRQManager>>> = SingleThreaded(RefCell::new(None));

fn get_manager() -> RefMut<'static, IRQManager> {
    let m = MANAGER.get().borrow_mut();
    if (*m).is_none() {
        *m = Some(IRQManager::new().unwrap());
    }
    RefMut::map(m, |b| &mut b.unwrap())
}

pub fn mainloop() {
    let m = &mut *get_manager();
    m.mainloop();
}

pub fn request(irq: u32) -> core::result::Result<IRQ<'static>, KError> {
    let m = &mut *get_manager();
    m.request(irq)
}
