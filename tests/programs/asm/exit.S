# Expected: exit code 42, no output.

.globl _start
_start:
    li a0, 42
    li a7, 93       # exit
    ecall
