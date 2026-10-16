# Error case: stops with a breakpoint error at pc 0x4.
# Expected: Err(Breakpoint(4)), no output (the interpreter exits with 1).

.globl _start
_start:
    nop
    ebreak
    li a7, 93       # never reached
    ecall
