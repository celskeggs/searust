use ::kobject::*;
use ::core;
use ::crust;
use ::mantle;
use ::mantle::KError;

pub struct IRQControl {
    cap: Cap
}

impl IRQControl {
    pub fn get(&self, irq: u32, output_slot: CapSlot) -> core::result::Result<IRQHandler, KError> {
        let err = mantle::irqcontrol_get(self.cap.peek_index(), irq, crust::ROOT_SLOT, output_slot.peek_index(), crust::ROOT_BITS);
        if err.is_err() {
            Err(err)
        } else {
            Ok(IRQHandler { cap: output_slot.assert_populated() })
        }
    }
}

pub struct IRQHandler {
    cap: Cap
}

impl IRQHandler {
    pub fn ack(&self) -> core::result::Result<(), KError> {
        mantle::irqhandler_ack(self.cap.peek_index()).to_result()
    }

    pub fn clear(&self) -> core::result::Result<(), KError> {
        mantle::irqhandler_clear(self.cap.peek_index()).to_result()
    }

    pub fn set_notification(&self, notification: &Notification) -> core::result::Result<(), KError> {
        mantle::irqhandler_set_notification(self.cap.peek_index(), notification.peek_index()).to_result()
    }
}
