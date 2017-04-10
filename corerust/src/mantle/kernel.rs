use ::core;

// A file of relevant constants and structures from libsel4

pub const PAGE_4K_BITS: u8 = 12;
pub const PAGE_4K_SIZE: usize = 1 << PAGE_4K_BITS;
pub const PAGE_2M_BITS: u8 = 21;
pub const PAGE_2M_SIZE: usize = 1 << PAGE_2M_BITS;

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum KError {
    // TODO: make sure this stays synchronized
    NoError = 0,
    InvalidArgument = 1,
    InvalidCapability = 2,
    IllegalOperation = 3,
    RangeError = 4,
    AlignmentError = 5,
    FailedLookup = 6,
    TruncatedMessage = 7,
    DeleteFirst = 8,
    RevokeFirst = 9,
    NotEnoughMemory = 10,
    UnknownError = 11,
    // aka NumErrors
}

impl KError {
    pub fn from_code(code: u32) -> KError {
        match code {
            0 => KError::NoError,
            1 => KError::InvalidArgument,
            2 => KError::InvalidCapability,
            3 => KError::IllegalOperation,
            4 => KError::RangeError,
            5 => KError::AlignmentError,
            6 => KError::FailedLookup,
            7 => KError::TruncatedMessage,
            8 => KError::DeleteFirst,
            9 => KError::RevokeFirst,
            10 => KError::NotEnoughMemory,
            _  => KError::UnknownError
        }
    }

    pub fn to_result(&self) -> core::result::Result<(), KError> {
        if self.is_error() {
            Err(*self)
        } else {
            Ok(())
        }
    }

    pub fn is_error(&self) -> bool {
        self != &KError::NoError
    }

    pub fn is_okay(&self) -> bool {
        self == &KError::NoError
    }
}

#[repr(usize)]
pub enum ObjectType {
    UntypedObject = 0,
    TCBObject = 1,
    EndpointObject = 2,
    NotificationObject = 3,
    CapTableObject = 4,
    X86PDPTObject = 5,
    X64PML4Object = 6,
    X864K = 7,
    X86LargePageObject = 8,
    X86PageTableObject = 9,
    X86PageDirectoryObject = 10,
}

#[repr(C, packed)]
pub struct BootInfo {
    pub extra_len: usize,
    pub node_id: usize,
    pub num_nodes: usize,
    pub num_iopt_levels: usize,
    pub ipc_buffer: *mut IPCBuffer,
    pub empty: SlotRegion,
    pub shared_frames: SlotRegion,
    pub user_image_frames: SlotRegion,
    pub user_image_paging: SlotRegion,
    pub io_space_caps: SlotRegion,
    pub extra_bi_pages: SlotRegion,
    pub init_thread_cnode_size_bits: u8,
    pub init_thread_domain: usize,
    pub arch_info: usize,
    pub untyped: SlotRegion,
    pub untyped_list: [UntypedDesc; 230usize],
}

#[repr(C)]
pub struct IPCBuffer {
    pub tag: usize,
    pub msg: [usize; 120],
    pub user_data: usize,
    pub caps_or_badges: [usize; 3],
    pub receive_cnode: usize,
    pub receive_index: usize,
    pub receive_depth: usize,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct SlotRegion {
    pub start: usize,
    pub end: usize,
}

impl core::fmt::Display for SlotRegion {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "[{}, {})", self.start, self.end)
    }
}

pub const SYS_CALL: isize = -1;
pub const SYS_REPLY_RECV: isize = -2;
pub const SYS_SEND: isize = -3;
pub const SYS_NBSEND: isize = -4;
pub const SYS_RECV: isize = -5;
pub const SYS_REPLY: isize = -6;
pub const SYS_YIELD: isize = -7;
pub const SYS_NBRECV: isize = -8;
pub const SYS_DEBUG_PUTCHAR: isize = -9;
pub const SYS_DEBUG_HALT: isize = -10;
pub const SYS_DEBUG_CAPIDENTIFY: isize = -11;
pub const SYS_DEBUG_CAPSNAPSHOT: isize = -12;
pub const SYS_DEBUG_NAMETHREAD: isize = -13;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct UntypedDesc {
    pub paddr: usize,
    padding1: u8,
    padding2: u8,
    pub size_bits: u8,
    pub is_device: u8,
}

pub const LOOKUP_FAILURE_NO_FAILURE: usize = 0;
pub const LOOKUP_FAILURE_INVALID_ROOT: usize = 1;
pub const LOOKUP_FAILURE_MISSING_CAPABILITY: usize = 2;
pub const LOOKUP_FAILURE_DEPTH_MISMATCH: usize = 3;
pub const LOOKUP_FAILURE_GUARD_MISMATCH: usize = 4;

pub const CAP_NULL: u32 = 0;
pub const CAP_INIT_TCB: u32 = 1;
pub const CAP_INIT_CNODE: u32 = 2;
pub const CAP_INIT_VSPACE: u32 = 3;
pub const CAP_INIT_IRQCONTROL: u32 = 4;
pub const CAP_INIT_ASIDCONTROL: u32 = 5;
pub const CAP_INIT_ASIDPOOL: u32 = 6; // for initial thread
pub const CAP_INIT_IOPORT: u32 = 7;
pub const CAP_INIT_IOSPACE: u32 = 8;
pub const CAP_INIT_BOOTINFO_FRAME: u32 = 9;
pub const CAP_INIT_IPCBUFFER: u32 = 10;
pub const CAP_INIT_DOMAIN: u32 = 11;
pub const CAP_INIT_COUNT: u32 = 12;

pub type MessageTag = u32;

pub const TAG_INVALID_INVOCATION: MessageTag = 0;
pub const TAG_UNTYPED_RETYPE: MessageTag = 1;
pub const TAG_TCB_READ_REGISTERS: MessageTag = 2;
pub const TAG_TCB_WRITE_REGISTERS: MessageTag = 3;
pub const TAG_TCB_COPY_REGISTERS: MessageTag = 4;
pub const TAG_TCB_CONFIGURE: MessageTag = 5;
pub const TAG_TCB_SET_PRIORITY: MessageTag = 6;
pub const TAG_TCB_SET_MC_PRIORITY: MessageTag = 7;
pub const TAG_TCB_SET_IPC_BUFFER: MessageTag = 8;
pub const TAG_TCB_SET_SPACE: MessageTag = 9;
pub const TAG_TCB_SUSPEND: MessageTag = 10;
pub const TAG_TCB_RESUME: MessageTag = 11;
pub const TAG_TCB_BIND_NOTIFICATION: MessageTag = 12;
pub const TAG_TCB_UNBIND_NOTIFICATION: MessageTag = 13;
pub const TAG_CNODE_REVOKE: MessageTag = 14;
pub const TAG_CNODE_DELETE: MessageTag = 15;
pub const TAG_CNODE_CANCEL_BADGED_SENDS: MessageTag = 16;
pub const TAG_CNODE_COPY: MessageTag = 17;
pub const TAG_CNODE_MINT: MessageTag = 18;
pub const TAG_CNODE_MOVE: MessageTag = 19;
pub const TAG_CNODE_MUTATE: MessageTag = 20;
pub const TAG_CNODE_ROTATE: MessageTag = 21;
pub const TAG_CNODE_SAVE_CALLER: MessageTag = 22;
pub const TAG_IRQ_ISSUE_IRQ_HANDLER: MessageTag = 23;
pub const TAG_IRQ_ACK_IRQ: MessageTag = 24;
pub const TAG_IRQ_SET_IRQ_HANDLER: MessageTag = 25;
pub const TAG_IRQ_CLEAR_IRQ_HANDLER: MessageTag = 26;
pub const TAG_DOMAINSET_SET: MessageTag = 27;

pub const TAG_X86_PDPT_MAP: MessageTag = 28;
pub const TAG_X86_PDPT_UNMAP: MessageTag = 29;

pub const TAG_X86_PAGEDIRECTORY_MAP: MessageTag = 30;
pub const TAG_X86_PAGEDIRECTORY_UNMAP: MessageTag = 31;
pub const TAG_X86_PAGEDIRECTORY_GET_STATUS_BITS: MessageTag = 32;
pub const TAG_X86_PAGETABLE_MAP: MessageTag = 33;
pub const TAG_X86_PAGETABLE_UNMAP: MessageTag = 34;
pub const TAG_X86_IOPAGETABLE_MAP: MessageTag = 35;
pub const TAG_X86_IOPAGETABLE_UNMAP: MessageTag = 36;
pub const TAG_X86_PAGE_MAP: MessageTag = 37;
pub const TAG_X86_PAGE_REMAP: MessageTag = 38;
pub const TAG_X86_PAGE_UNMAP: MessageTag = 39;
pub const TAG_X86_PAGE_MAP_IO: MessageTag = 40;
pub const TAG_X86_PAGE_GET_ADDRESS: MessageTag = 41;
pub const TAG_X86_ASIDCONTROL_MAKEPOOL: MessageTag = 42;
pub const TAG_X86_ASIDPOOL_ASSIGN: MessageTag = 43;
pub const TAG_X86_IOPORT_IN8: MessageTag = 44;
pub const TAG_X86_IOPORT_IN16: MessageTag = 45;
pub const TAG_X86_IOPORT_IN32: MessageTag = 46;
pub const TAG_X86_IOPORT_OUT8: MessageTag = 47;
pub const TAG_X86_IOPORT_OUT16: MessageTag = 48;
pub const TAG_X86_IOPORT_OUT32: MessageTag = 49;
pub const TAG_X86_IRQ_ISSUE_IRQHANDLER_IOAPIC: MessageTag = 50;
pub const TAG_X86_IRQ_ISSUE_IRQHANDLER_MSI: MessageTag = 51;

pub type MessageInfo = u32;

pub fn messageinfo_new(label: u32, caps_unwrapped: u8, extra_caps: u8, length: u8) -> MessageInfo {
    /* fail if user has passed bits that we will override */
    assert!((label & !0xfffffu32) == 0);
    assert!((caps_unwrapped & !0x7u8) == 0);
    assert!((extra_caps & !0x3u8) == 0);
    assert!((length & !0x7fu8) == 0);

    (((label as u32) & 0xfffffu32) << 12) | (((caps_unwrapped as u32) & 0x7u32) << 9)
        | (((extra_caps as u32) & 0x3u32) << 7) | ((length as u32) & 0x7fu32)
}

pub fn messageinfo_get_label(info: MessageInfo) -> u32 {
    (info & 0xfffff000u32) >> 12
}
