use ::core::marker::Sync;

// Used for now while stuff is single-threaded.
pub struct SingleThreaded<T>(pub T);

impl<T> SingleThreaded<T> {
    pub fn take(self) -> T {
        self.0
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }

    pub fn get(&self) -> &T {
        &self.0
    }
}

unsafe impl<T> Sync for SingleThreaded<T> {}