#define __ASM__
#include <sel4/arch/constants.h>

.global _start
.extern premain
.text

_start:
    leaq _stack_top, %rsp

    /* get into rust. */
    pushq %rbx /* rbx primarily used for seL4_BootInfo pointer in root process */
    call rust_main

_fail:
    jmp _fail

.bss
.align 4

_stack_bottom:
    .space 65536
_stack_top:
