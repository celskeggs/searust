.global _start
.extern premain
.text

_start:
    leaq _stack_top, %rsp

    /* get into rust. */
/*    movq %rbx, %rdi /* seL4_BootInfo pointer */
    call rust_main

_fail:
    jmp _fail

.bss
.align 4

_stack_bottom:
    .space 65536
_stack_top:
