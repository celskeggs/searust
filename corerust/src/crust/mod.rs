// a crust is full of resources

pub mod start;
pub mod capalloc;
pub mod vspace;

// TODO: find a better place
pub const ROOT_SLOT: usize = ::mantle::kernel::CAP_INIT_CNODE;
pub const ROOT_PAGEDIR: usize = ::mantle::kernel::CAP_INIT_VSPACE;
pub const ROOT_BITS: usize = 64; // TODO: maybe this should be 32?
