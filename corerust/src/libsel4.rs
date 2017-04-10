pub type seL4_Word = usize;
pub type seL4_Uint8 = u8;
pub type seL4_NodeId = usize;
pub type seL4_SlotPos = usize;
pub type seL4_Syscall_ID = isize;
pub type seL4_CPtr = usize;
pub type seL4_Domain = usize;
pub type seL4_MessageInfo = usize;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct seL4_SlotRegion {
    pub start: seL4_SlotPos,
    pub end: seL4_SlotPos,
}

pub const seL4_Syscall_ID_seL4_SysCall: seL4_Syscall_ID = -1;
pub const seL4_Syscall_ID_seL4_SysReplyRecv: seL4_Syscall_ID = -2;
pub const seL4_Syscall_ID_seL4_SysSend: seL4_Syscall_ID = -3;
pub const seL4_Syscall_ID_seL4_SysNBSend: seL4_Syscall_ID = -4;
pub const seL4_Syscall_ID_seL4_SysRecv: seL4_Syscall_ID = -5;
pub const seL4_Syscall_ID_seL4_SysReply: seL4_Syscall_ID = -6;
pub const seL4_Syscall_ID_seL4_SysYield: seL4_Syscall_ID = -7;
pub const seL4_Syscall_ID_seL4_SysNBRecv: seL4_Syscall_ID = -8;
pub const seL4_Syscall_ID_seL4_SysDebugPutChar: seL4_Syscall_ID = -9;
pub const seL4_Syscall_ID_seL4_SysDebugHalt: seL4_Syscall_ID = -10;
pub const seL4_Syscall_ID_seL4_SysDebugCapIdentify: seL4_Syscall_ID = -11;
pub const seL4_Syscall_ID_seL4_SysDebugSnapshot: seL4_Syscall_ID = -12;
pub const seL4_Syscall_ID_seL4_SysDebugNameThread: seL4_Syscall_ID = -13;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct seL4_UntypedDesc {
    pub paddr: seL4_Word,
    pub padding1: seL4_Uint8,
    pub padding2: seL4_Uint8,
    pub sizeBits: seL4_Uint8,
    pub isDevice: seL4_Uint8,
}

#[repr(C, packed)]
pub struct seL4_BootInfo {
    pub extraLen: seL4_Word,
    pub nodeID: seL4_NodeId,
    pub numNodes: seL4_Word,
    pub numIOPTLevels: seL4_Word,
    pub ipcBuffer: *mut seL4_IPCBuffer,
    pub empty: seL4_SlotRegion,
    pub sharedFrames: seL4_SlotRegion,
    pub userImageFrames: seL4_SlotRegion,
    pub userImagePaging: seL4_SlotRegion,
    pub ioSpaceCaps: seL4_SlotRegion,
    pub extraBIPages: seL4_SlotRegion,
    pub initThreadCNodeSizeBits: seL4_Uint8,
    pub initThreadDomain: seL4_Domain,
    pub archInfo: seL4_Word,
    pub untyped: seL4_SlotRegion,
    pub untypedList: [seL4_UntypedDesc; 230usize],
}

pub const seL4_LookupFailureType_seL4_NoFailure: seL4_LookupFailureType = 0;
pub const seL4_LookupFailureType_seL4_InvalidRoot: seL4_LookupFailureType = 1;
pub const seL4_LookupFailureType_seL4_MissingCapability: seL4_LookupFailureType = 2;
pub const seL4_LookupFailureType_seL4_DepthMismatch: seL4_LookupFailureType = 3;
pub const seL4_LookupFailureType_seL4_GuardMismatch: seL4_LookupFailureType = 4;

pub type seL4_LookupFailureType = usize;

pub const invocation_label_InvalidInvocation: invocation_label = 0;
pub const invocation_label_UntypedRetype: invocation_label = 1;
pub const invocation_label_TCBReadRegisters: invocation_label = 2;
pub const invocation_label_TCBWriteRegisters: invocation_label = 3;
pub const invocation_label_TCBCopyRegisters: invocation_label = 4;
pub const invocation_label_TCBConfigure: invocation_label = 5;
pub const invocation_label_TCBSetPriority: invocation_label = 6;
pub const invocation_label_TCBSetMCPriority: invocation_label = 7;
pub const invocation_label_TCBSetIPCBuffer: invocation_label = 8;
pub const invocation_label_TCBSetSpace: invocation_label = 9;
pub const invocation_label_TCBSuspend: invocation_label = 10;
pub const invocation_label_TCBResume: invocation_label = 11;
pub const invocation_label_TCBBindNotification: invocation_label = 12;
pub const invocation_label_TCBUnbindNotification: invocation_label = 13;
pub const invocation_label_CNodeRevoke: invocation_label = 14;
pub const invocation_label_CNodeDelete: invocation_label = 15;
pub const invocation_label_CNodeCancelBadgedSends: invocation_label = 16;
pub const invocation_label_CNodeCopy: invocation_label = 17;
pub const invocation_label_CNodeMint: invocation_label = 18;
pub const invocation_label_CNodeMove: invocation_label = 19;
pub const invocation_label_CNodeMutate: invocation_label = 20;
pub const invocation_label_CNodeRotate: invocation_label = 21;
pub const invocation_label_CNodeSaveCaller: invocation_label = 22;
pub const invocation_label_IRQIssueIRQHandler: invocation_label = 23;
pub const invocation_label_IRQAckIRQ: invocation_label = 24;
pub const invocation_label_IRQSetIRQHandler: invocation_label = 25;
pub const invocation_label_IRQClearIRQHandler: invocation_label = 26;
pub const invocation_label_DomainSetSet: invocation_label = 27;
pub const invocation_label_nInvocationLabels: invocation_label = 28;

pub type invocation_label = u32;

pub const seL4_CapNull: _bindgen_ty_6 = 0;
pub const seL4_CapInitThreadTCB: _bindgen_ty_6 = 1;
pub const seL4_CapInitThreadCNode: _bindgen_ty_6 = 2;
pub const seL4_CapInitThreadVSpace: _bindgen_ty_6 = 3;
pub const seL4_CapIRQControl: _bindgen_ty_6 = 4;
pub const seL4_CapASIDControl: _bindgen_ty_6 = 5;
pub const seL4_CapInitThreadASIDPool: _bindgen_ty_6 = 6;
pub const seL4_CapIOPort: _bindgen_ty_6 = 7;
pub const seL4_CapIOSpace: _bindgen_ty_6 = 8;
pub const seL4_CapBootInfoFrame: _bindgen_ty_6 = 9;
pub const seL4_CapInitThreadIPCBuffer: _bindgen_ty_6 = 10;
pub const seL4_CapDomain: _bindgen_ty_6 = 11;
pub const seL4_NumInitialCaps: _bindgen_ty_6 = 12;

pub type _bindgen_ty_6 = u32;

pub const arch_invocation_label_X86PDPTMap: arch_invocation_label = 28;
pub const arch_invocation_label_X86PDPTUnmap: arch_invocation_label = 29;
pub const arch_invocation_label_X86PageDirectoryMap: arch_invocation_label = 30;
pub const arch_invocation_label_X86PageDirectoryUnmap: arch_invocation_label = 31;
pub const arch_invocation_label_X86PageDirectoryGetStatusBits: arch_invocation_label = 32;
pub const arch_invocation_label_X86PageTableMap: arch_invocation_label = 33;
pub const arch_invocation_label_X86PageTableUnmap: arch_invocation_label = 34;
pub const arch_invocation_label_X86IOPageTableMap: arch_invocation_label = 35;
pub const arch_invocation_label_X86IOPageTableUnmap: arch_invocation_label = 36;
pub const arch_invocation_label_X86PageMap: arch_invocation_label = 37;
pub const arch_invocation_label_X86PageRemap: arch_invocation_label = 38;
pub const arch_invocation_label_X86PageUnmap: arch_invocation_label = 39;
pub const arch_invocation_label_X86PageMapIO: arch_invocation_label = 40;
pub const arch_invocation_label_X86PageGetAddress: arch_invocation_label = 41;
pub const arch_invocation_label_X86ASIDControlMakePool: arch_invocation_label = 42;
pub const arch_invocation_label_X86ASIDPoolAssign: arch_invocation_label = 43;
pub const arch_invocation_label_X86IOPortIn8: arch_invocation_label = 44;
pub const arch_invocation_label_X86IOPortIn16: arch_invocation_label = 45;
pub const arch_invocation_label_X86IOPortIn32: arch_invocation_label = 46;
pub const arch_invocation_label_X86IOPortOut8: arch_invocation_label = 47;
pub const arch_invocation_label_X86IOPortOut16: arch_invocation_label = 48;
pub const arch_invocation_label_X86IOPortOut32: arch_invocation_label = 49;
pub const arch_invocation_label_X86IRQIssueIRQHandlerIOAPIC: arch_invocation_label = 50;
pub const arch_invocation_label_X86IRQIssueIRQHandlerMSI: arch_invocation_label = 51;
pub const arch_invocation_label_nArchInvocationLabels: arch_invocation_label = 52;

pub type arch_invocation_label = u32;

#[repr(C)]
pub struct seL4_IPCBuffer {
    pub tag: seL4_MessageInfo,
    pub msg: [seL4_Word; 120usize],
    pub userData: seL4_Word,
    pub caps_or_badges: [seL4_Word; 3usize],
    pub receiveCNode: seL4_CPtr,
    pub receiveIndex: seL4_CPtr,
    pub receiveDepth: seL4_Word,
}
