use ::memory;
use ::core;

// TODO: expand to support sizes larger than 2032, which will depend on a better malloc

pub const MAX_STRING_LEN: usize = memory::alloc::MAX_ALLOC_LEN - 8;

pub struct VarStr<'a> {
    memory: &'a [u8]
}

impl<'a> VarStr<'a> {
    pub fn as_str(&self) -> &'a str {
        core::str::from_utf8(self.memory).unwrap()
    }
}

impl<'a> Drop for VarStr<'a> {
    fn drop(&mut self) {
        unsafe { memory::malloc::free(self.memory.as_ptr() as *mut u8) }
    }
}

pub struct StringBuilder {
    memory: [u8; MAX_STRING_LEN],
    index: usize,
    was_truncated: bool
}

impl StringBuilder {
    pub fn new() -> StringBuilder {
        StringBuilder { index: 0, was_truncated: false, memory: [0; MAX_STRING_LEN] }
    }

    pub fn is_truncated(&self) -> bool {
        self.was_truncated
    }

    pub fn add_u8(&mut self, b: u8) {
        if self.index < MAX_STRING_LEN {
            self.memory[self.index] = b;
            self.index += 1;
        } else {
            self.was_truncated = true; // oops!
        }
    }

    pub fn add_bytes(&mut self, b: &[u8]) {
        if self.index + b.len() <= MAX_STRING_LEN {
            for i in 0 .. b.len() {
                self.memory[self.index + i] = b[i]
            }
            self.index += b.len()
        } else {
            let remain = MAX_STRING_LEN - b.len();
            for i in 0 .. remain {
                self.memory[self.index + i] = b[i]
            }
            self.index += remain;
            self.was_truncated = true;
        }
    }

    pub fn add_str(&mut self, b: &str) {
        self.add_bytes(b.as_bytes())
    }

    pub fn add_char(&mut self, b: char) {
        let mut tbuf: [u8; 4] = [0; 4];
        self.add_str(b.encode_utf8(&mut tbuf))
    }

    pub fn to_str(&self) -> Option<VarStr<'static>> {
        if let Some(ptr) = memory::malloc::malloc(self.index) {
            unsafe {
                core::ptr::copy_nonoverlapping(self.memory.as_ptr(), ptr, self.index);
                Some(VarStr { memory: core::slice::from_raw_parts(ptr, self.index) })
            }
        } else {
            None
        }
    }
}
