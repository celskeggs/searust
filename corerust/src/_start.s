.global _start
.extern premain.
.extern __executable_start
.text

_start:
    leaq _stack_top, %rsp

    /* get into rust. */
    movq $__executable_start, %rsi
    call rust_main

_fail:
    jmp _fail

.bss
.align 4

_stack_bottom:
    .space 65536
_stack_top:
