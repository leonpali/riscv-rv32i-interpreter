# Expected: prints "Hello, world!\n", exit code 0.

.include "check.inc"

.section .rodata
msg:
    .ascii "Hello, world!\n"
.equ MSG_LEN, . - msg

.text
.globl _start
_start:
    li a0, 1                # fd = stdout
    la a1, msg              # buf
    li a2, MSG_LEN          # len
    li a7, 64               # write
    ecall
    exit 0
