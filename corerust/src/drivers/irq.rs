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

pub const IRQ_MAX: u8 = 32;

struct IRQManager {
    irqcontrol: IRQControl,
    notification: Notification,
    callbacks: [RefCell<Option<FnMut()>>; IRQ_MAX as usize]
}

pub struct IRQ<'a> {
    irqhandler: IRQHandler,
    manager: &'a IRQManager,
    irq: u8
}

impl IRQManager {
    fn new() -> IRQManager {
        let irqc = IRQControl::from_cap(CapSlot::from_index(mantle::kernel::CAP_INIT_IRQCONTROL).assert_populated());
        IRQManager { irqcontrol: irqc, notification: memory::smalluntyped::allocate_notification(), callbacks: [None; IRQ_MAX as usize] }
    }

    fn request<'a>(&'a self, irq: u8) -> core::result::Result<IRQ<'a>, KError> {
        assert!(irq < IRQ_MAX); // make sure it fits in a notification word
        let cslot = crust::capalloc::allocate_cap_slot()?;
        match self.irqc.get(irq, cslot) {
            Ok(irqhandler) => {
                irqhandler.set_notification(self.notification);
                Ok(IRQ { irqhandler, manager: self, irq })
            }
            Err((cslot, err)) => {
                crust::capalloc::free_cap_slot(cslot);
                Err(err)
            }
        }
    }

    fn on_bit(&mut self, bit: u32) {
        if let &Some(ref cb) = self.callbacks[bit].borrow() {
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

    fn set_callback(&self, irq: u8, cb: FnMut()) {
        assert!(self.callbacks[irq].is_none());
        self.callbacks[irq] = Some(cb);
    }

    fn clear_callback(&self, irq: u8) {
        assert!(self.callbacks[irq].is_some());
        self.callbacks[irq] = None;
    }
}

impl<'a> IRQ<'a> {
    fn free(self) {
        self.clear_cb();
        assert!(self.irqhandler.clear().is_ok());
        crust::capalloc::free_cap_slot(self.irqhandler.free())
    }

    fn ack(&self) -> KError {
        self.irqhandler.ack()
    }

    fn set_cb(&self, cb: FnMut()) {
        self.manager.set_callback(self.irq, cb)
    }

    fn clear_cb(&self) {
        self.manager.clear_callback(self.irq);
    }
}

static MANAGER: SingleThreaded<RefCell<Option<IRQManager>>> = SingleThreaded(RefCell::new(None));

fn get_manager() -> RefMut<'static, IRQManager> {
    let m = MANAGER.get().borrow_mut();
    if (*m).is_none() {
        *m = Some(IRQManager::new());
    }
    RefMut::map(m, |b| b.unwrap())
}

pub fn mainloop() {
    let m = &mut *get_manager();
    m.mainloop();
}

pub fn request(irq: u8) -> core::result::Result<IRQ<'static>, KError> {
    let m = &mut *get_manager();
    m.request(irq)
}
