use ::core;
use ::memory::alloc;

pub struct Box<T> {
    ptr: *mut T
}

impl<T> Box<T> {
    pub fn newchk(x: T) -> core::result::Result<Box<T>, T> {
        match alloc::alloc_type(x) {
            Ok(ptr) => {
                Ok(Box { ptr })
            }
            Err(x) => {
                Err(x)
            }
        }
    }

    pub fn new(x: T) -> Box<T> {
        match alloc::alloc_type(x) {
            Ok(ptr) => {
                Box { ptr }
            }
            Err(_) => {
                panic!("could not allocate memory for box");
            }
        }
    }
}

impl<T> Box<T> {
    pub unsafe fn from_raw(raw: *mut T) -> Box<T> {
        Box { ptr: raw }
    }

    pub fn into_raw(b: Box<T>) -> *mut T {
        let out = b.ptr;
        core::mem::forget(b);
        out
    }
}

impl<T> core::borrow::BorrowMut<T> for Box<T> {
    fn borrow_mut(&mut self) -> &mut T {
        unsafe {
            &mut *self.ptr
        }
    }
}

impl<T> core::borrow::Borrow<T> for Box<T> {
    fn borrow(&self) -> &T {
        unsafe {
            &*self.ptr
        }
    }
}

impl<T: core::fmt::Display> core::fmt::Display for Box<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        let deref: &T = &self;
        write!(f, "{}", deref)
    }
}

impl<T> core::ops::DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            &mut *self.ptr
        }
    }
}

impl<T> core::ops::Deref for Box<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            &*self.ptr
        }
    }
}

impl<T> core::convert::AsRef<T> for Box<T> {
    fn as_ref(&self) -> &T {
        unsafe {
            &*self.ptr
        }
    }
}

impl<T> core::convert::AsMut<T> for Box<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe {
            &mut *self.ptr
        }
    }
}

impl<T> core::convert::From<T> for Box<T> {
    fn from(t: T) -> Box<T> {
        Box::new(t)
    }
}

impl<T> core::ops::Drop for Box<T> {
    fn drop(&mut self) {
        core::mem::drop(unsafe {
            alloc::dealloc_type(self.ptr)
        });
    }
}
