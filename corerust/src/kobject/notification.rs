use ::mantle;
use ::kobject::*;

pub struct Notification {
    cap: Cap,
    parent: Untyped
}

impl Notification {
    pub fn from_retyping(cap: Cap, parent: Untyped) -> Notification {
        Notification { cap, parent }
    }

    pub fn free(self) -> (Untyped, CapSlot) {
        (self.parent, self.cap.delete())
    }

    pub fn signal(&self) {
        mantle::signal(self.cap.peek_index())
    }

    pub fn wait(&self) -> usize {
        mantle::wait(self.cap.peek_index())
    }

    pub fn poll(&self) -> usize {
        mantle::poll(self.cap.peek_index())
    }

    pub fn peek_index(&self) -> usize {
        self.cap.peek_index()
    }

    // TODO: badging
}
