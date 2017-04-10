// a crust is full of resources

pub mod start;
mod capalloc;
pub mod device;
pub mod vspace;

// TODO: find a better place
pub const ROOT_SLOT: usize = ::mantle::kernel::CAP_INIT_CNODE as usize;
pub const ROOT_PAGEDIR: usize = ::mantle::kernel::CAP_INIT_VSPACE as usize;
pub const ROOT_BITS: usize = 64; // TODO: maybe this should be 32?
