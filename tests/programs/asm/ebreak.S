# Error case: stops with a breakpoint error at pc 0x80000004.
# Expected: Err(Breakpoint(0x80000004)), no output (the interpreter exits with 1).

.globl _start
_start:
    nop
    ebreak
    li a7, 93       # never reached
    ecall
