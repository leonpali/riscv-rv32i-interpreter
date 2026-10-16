# Recursive Fibonacci: uses call/ret (jal/jalr) and the stack (sw/lw).
# Expected: exit code 55 (fib(10)), no output.

.globl _start
_start:
    li sp, 0x10000          # the loader doesn't set up a stack yet
    li a0, 10
    call fib
    li a7, 93               # exit(fib(10))
    ecall

# a0 = n  ->  a0 = fib(n)
fib:
    li t0, 2
    blt a0, t0, 1f          # fib(0) = 0, fib(1) = 1

    addi sp, sp, -12
    sw ra, 8(sp)
    sw s0, 4(sp)
    sw s1, 0(sp)

    mv s0, a0
    addi a0, s0, -1
    call fib                # fib(n - 1)
    mv s1, a0
    addi a0, s0, -2
    call fib                # fib(n - 2)
    add a0, a0, s1

    lw s1, 0(sp)
    lw s0, 4(sp)
    lw ra, 8(sp)
    addi sp, sp, 12
1:
    ret
