use ::mantle::kernel;

// fundamental kernel I/O functions

const CAPS_OR_BADGES_LEN: usize = 3;
const MSG_LEN: usize = 120;

#[cfg(target_arch = "x86_64")]
pub fn set_cap(i: u32, cptr: usize) {
    assert!((i as usize) < CAPS_OR_BADGES_LEN);
    unsafe {
        asm!(
            // 8 comes from size of usize; 976 comes from caps_or_badges offset
            "movq $0, %gs:976(,$1,8)"
            : /* no outputs */
            : "r" (cptr),
            "r" (i)
            : "memory"
            : "volatile"
        );
    }
}


#[cfg(target_arch = "x86_64")]
pub fn set_mr(i: u32, mr: usize) {
    assert!((i as usize) < MSG_LEN);
    unsafe {
        asm!(
            // latter 8 comes from size of usize; earlier 8 comes from msg offset
            "movq $0, %gs:8(,$1,8)"
            : /* no outputs */
            : "r" (mr),
            "r" (i)
            : "memory"
            : "volatile"
        );
    }
}

#[cfg(target_arch = "x86_64")]
pub fn get_mr(i: u32) -> usize {
    assert!((i as usize) < MSG_LEN);
    let mr_out;
    unsafe {
        asm!(
            // latter 8 comes from size of usize; earlier 8 comes from msg offset
            "movq %gs:8(,$1,8), $0"
            : "=r" (mr_out)
            : "r" (i)
            : "memory"
            : "volatile"
        );
    }
    mr_out
}

#[cfg(target_arch = "x86_64")]
unsafe fn x64_sys_send(syscall: isize, dest: u64, info: u64, mr0: u64, mr1: u64, mr2: u64, mr3: u64) {
    asm!(
		"movq %rsp, %rbx\nsyscall\nmovq %rbx, %rsp\n"
		:: "{rdx}"(syscall),
		"{rdi}"(dest),
		"{rsi}"(info),
		"{r10}"(mr0),
		"{r8}"(mr1),
		"{r9}"(mr2),
		"{r15}"(mr3)
		: "rcx", "rbx", "r11", "memory"
		: "volatile"
	);
}

#[cfg(target_arch = "x86_64")]
unsafe fn x64_sys_reply(syscall: isize, info: u64, mr0: u64, mr1: u64, mr2: u64, mr3: u64) {
    asm!(
		"movq %rsp, %rbx\nsyscall\nmovq %rbx, %rsp\n"
		:: "{rdx}"(syscall),
		"{rsi}"(info),
		"{r10}"(mr0),
		"{r8}"(mr1),
		"{r9}"(mr2),
		"{r15}"(mr3)
		: "rcx", "rbx", "r11", "memory"
		: "volatile"
	);
}

#[cfg(target_arch = "x86_64")]
unsafe fn x64_sys_send_null(syscall: isize, dest: u64, info: u64) {
    asm!(
		"movq %rsp, %rbx\nsyscall\nmovq %rbx, %rsp\n"
		:: "{rdx}"(syscall),
		"{rdi}"(dest),
		"{rsi}"(info)
		: "rcx", "rbx", "r11", "memory"
		: "volatile"
	);
}

#[cfg(target_arch = "x86_64")]
unsafe fn x64_sys_recv(syscall: isize, src: u64) -> (u64, u64, u64, u64, u64, u64) {
    let info_out;
    let dest_out;
    let mr0_out;
    let mr1_out;
    let mr2_out;
    let mr3_out;
    asm!(
		"movq %rsp, %rbx\nsyscall\nmovq %rbx, %rsp\n"
		: "={rsi}"(info_out),
		"={r10}"(mr0_out),
		"={r8}"(mr1_out),
		"={r9}"(mr2_out),
		"={r15}"(mr3_out),
		"={rdi}"(dest_out)
		: "{rdx}"(syscall),
		"{rdi}"(src)
		: "rcx", "rbx", "r11", "memory"
		: "volatile"
	);
    (dest_out, info_out, mr0_out, mr1_out, mr2_out, mr3_out)
}

#[cfg(target_arch = "x86_64")]
unsafe fn x64_sys_send_recv(syscall: isize, dest: u64, info: u64, mr0: u64, mr1: u64,
                            mr2: u64, mr3: u64) -> (u64, u64, u64, u64, u64, u64) {
    let info_out;
    let dest_out;
    let mr0_out;
    let mr1_out;
    let mr2_out;
    let mr3_out;
    asm!(
		"movq %rsp, %rbx\nsyscall\nmovq %rbx, %rsp\n"
		: "={rsi}"(info_out),
		"={r10}"(mr0_out),
		"={r8}"(mr1_out),
		"={r9}"(mr2_out),
		"={r15}"(mr3_out),
		"={rdi}"(dest_out)
		: "{rdx}"(syscall),
		"{rdi}"(dest),
		"{rsi}"(info),
		"{r10}"(mr0),
		"{r8}"(mr1),
		"{r9}"(mr2),
		"{r15}"(mr3)
		: "rcx", "rbx", "r11", "memory"
		: "volatile"
	);
    (dest_out, info_out, mr0_out, mr1_out, mr2_out, mr3_out)
}

#[cfg(target_arch = "x86_64")]
unsafe fn x64_sys_null(syscall: isize) {
    asm!(
		"movq %rsp, %rbx\nsyscall\nmovq %rbx, %rsp\n"
		:: "{rdx}"(syscall)
		: "rcx", "rbx", "rsi", "rdi", "r11", "memory"
		: "volatile"
	);
}

// "syscalls"

pub fn debug_put_char(c: u8) {
    unsafe {
        x64_sys_send_recv(kernel::SYS_DEBUG_PUTCHAR, c as u64, 0, 0, 0, 0, 0);
    }
}

pub unsafe fn send(dest: usize, info: kernel::MessageInfo) {
    x64_sys_send(kernel::SYS_SEND, dest, info, get_mr(0), get_mr(1), get_mr(2), get_mr(3))
}

pub unsafe fn send_with_mrs(dest: usize, info: kernel::MessageInfo, mr0: usize, mr1: usize, mr2: usize, mr3: usize) {
    x64_sys_send(kernel::SYS_SEND, dest, info, mr0, mr1, mr2, mr3)
}

pub unsafe fn nbsend(dest: usize, info: kernel::MessageInfo) {
    x64_sys_send(kernel::SYS_NBSEND, dest, info, get_mr(0), get_mr(1), get_mr(2), get_mr(3))
}

pub unsafe fn nbsend_with_mrs(dest: usize, info: kernel::MessageInfo, mr0: usize, mr1: usize, mr2: usize, mr3: usize) {
    x64_sys_send(kernel::SYS_NBSEND, dest, info, mr0, mr1, mr2, mr3)
}

pub unsafe fn reply(info: kernel::MessageInfo) {
    x64_sys_reply(kernel::SYS_REPLY, info, get_mr(0), get_mr(1), get_mr(2), get_mr(3))
}

pub unsafe fn reply_with_mrs(info: kernel::MessageInfo, mr0: usize, mr1: usize, mr2: usize, mr3: usize) {
    x64_sys_reply(kernel::SYS_REPLY, info, mr0, mr1, mr2, mr3)
}

pub fn signal(dest: usize) {
    unsafe {
        x64_sys_send_null(kernel::SYS_SEND, dest, kernel::messageinfo_new(0, 0, 0, 1))
    }
}

pub unsafe fn recv(src: usize) -> (kernel::MessageInfo, usize) {
    let (badge, info, mr0, mr1, mr2, mr3) = x64_sys_recv(kernel::SYS_RECV, src);
    set_mr(0, mr0);
    set_mr(1, mr1);
    set_mr(2, mr2);
    set_mr(3, mr3);
    (info, badge)
}

pub unsafe fn recv_with_mrs(src: usize) -> (kernel::MessageInfo, usize, usize, usize, usize, usize) {
    let (badge, info, mr0, mr1, mr2, mr3) = x64_sys_recv(kernel::SYS_RECV, src);
    (info, badge, mr0, mr1, mr2, mr3)
}

pub unsafe fn nbrecv(src: usize) -> (kernel::MessageInfo, usize) {
    let (badge, info, mr0, mr1, mr2, mr3) = x64_sys_recv(kernel::SYS_NBRECV, src);
    set_mr(0, mr0);
    set_mr(1, mr1);
    set_mr(2, mr2);
    set_mr(3, mr3);
    (info, badge)
}

pub unsafe fn call(dest: usize, msginfo: kernel::MessageInfo) -> kernel::MessageInfo {
    let mr0 = get_mr(0);
    let mr1 = get_mr(1);
    let mr2 = get_mr(2);
    let mr3 = get_mr(3);

    let (_, info_out, mr0_out, mr1_out, mr2_out, mr3_out) =
        x64_sys_send_recv(kernel::SYS_CALL, dest as u64, msginfo as u64, mr0 as u64, mr1 as u64, mr2 as u64, mr3 as u64);

    set_mr(0, mr0_out);
    set_mr(1, mr1_out);
    set_mr(2, mr2_out);
    set_mr(3, mr3_out);

    info_out as kernel::MessageInfo
}

pub unsafe fn call_with_mrs(dest: usize, msginfo: kernel::MessageInfo, mr0: usize, mr1: usize, mr2: usize, mr3: usize)
                            -> (kernel::MessageInfo, usize, usize, usize, usize) {
    let (_, info_out, mr0_out, mr1_out, mr2_out, mr3_out) =
        x64_sys_send_recv(kernel::SYS_CALL, dest as u64, msginfo as u64, mr0 as u64, mr1 as u64, mr2 as u64, mr3 as u64);

    (info_out as kernel::MessageInfo, mr0_out as usize, mr1_out as usize, mr2_out as usize, mr3_out as usize)
}

pub unsafe fn reply_recv(dest: usize, info: kernel::MessageInfo) -> (kernel::MessageInfo, usize) {
    let mr0 = get_mr(0);
    let mr1 = get_mr(1);
    let mr2 = get_mr(2);
    let mr3 = get_mr(3);

    let (_, info_out, mr0_out, mr1_out, mr2_out, mr3_out) =
        x64_sys_send_recv(kernel::SYS_REPLY_RECV, dest, info, mr0, mr1, mr2, mr3);

    set_mr(0, mr0_out);
    set_mr(1, mr1_out);
    set_mr(2, mr2_out);
    set_mr(3, mr3_out);

    (info_out as kernel::MessageInfo, info_out)
}

pub unsafe fn reply_recv_with_mrs(dest: usize, info: kernel::MessageInfo, mr0: usize, mr1: usize, mr2: usize, mr3: usize) -> (kernel::MessageInfo, usize, usize, usize, usize, usize) {
    let (_, info_out, mr0_out, mr1_out, mr2_out, mr3_out) =
        x64_sys_send_recv(kernel::SYS_REPLY_RECV, dest, info, mr0, mr1, mr2, mr3);

    (info_out as kernel::MessageInfo, info_out, mr0_out, mr1_out, mr2_out, mr3_out)
}

pub fn yield_() {
    unsafe {
        x64_sys_null(kernel::SYS_YIELD);
    }
}

// TODO: vmenter?

pub fn wait(src: usize) -> usize {
    unsafe {
        recv(src).1
    }
}

pub fn poll(src: usize) -> usize {
    unsafe {
        nbrecv(src)
    }
}
