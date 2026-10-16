# All branches (taken and not taken, signed vs unsigned), loops and jumps.
# Expected: exit code 0, no output. Any other code is the failing check.

.include "check.inc"

.globl _start
_start:
    li t0, -1                   # 0xffffffff
    li t1, 1

    # ---- beq / bne ----
    li a0, 1
    beq t0, t1, fail
    li a0, 2
    beq t0, t0, 1f
    j fail
1:
    li a0, 3
    bne t0, t0, fail
    li a0, 4
    bne t0, t1, 1f
    j fail
1:

    # ---- blt / bge (signed: -1 < 1) ----
    li a0, 5
    blt t1, t0, fail
    li a0, 6
    blt t0, t1, 1f
    j fail
1:
    li a0, 7
    bge t0, t1, fail
    li a0, 8
    bge t1, t0, 1f
    j fail
1:
    li a0, 9
    bge t1, t1, 1f              # equal counts as >=
    j fail
1:

    # ---- bltu / bgeu (unsigned: 0xffffffff > 1) ----
    li a0, 10
    bltu t0, t1, fail
    li a0, 11
    bltu t1, t0, 1f
    j fail
1:
    li a0, 12
    bgeu t1, t0, fail
    li a0, 13
    bgeu t0, t1, 1f
    j fail
1:
    li a0, 14
    bgeu t0, t0, 1f
    j fail
1:

    # ---- backward branch: loop 5 times ----
    li t2, 5
    li t3, 0
2:
    addi t3, t3, 1
    addi t2, t2, -1
    bnez t2, 2b
    check t3, 5, 15

    # ---- jal / jalr ----
    li t4, 0
    call set_t4                 # jal ra, set_t4 (returns with jalr)
    check t4, 77, 16
    li t4, 0
    la t5, set_t4
    jalr t5                     # indirect call through a register
    check t4, 77, 17

    exit 0

set_t4:
    li t4, 77
    ret

fail:
    li a7, 93
    ecall
