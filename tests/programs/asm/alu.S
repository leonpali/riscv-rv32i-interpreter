# Arithmetic, logic, shifts, comparisons, lui/auipc and x0.
# Expected: exit code 0, no output. Any other code is the failing check.

.include "check.inc"

.globl _start
_start:
    # ---- add / sub ----
    li t0, 5
    addi t1, t0, -10
    check t1, -5, 1
    li t0, 0x7fffffff
    addi t1, t0, 1              # overflow wraps
    check t1, 0x80000000, 2
    li t0, 3
    li t1, 4
    add t2, t0, t1
    check t2, 7, 3
    sub t2, t0, t1
    check t2, -1, 4

    # ---- logic ----
    li t0, 0xff00ff00
    li t1, 0x0ff00ff0
    and t2, t0, t1
    check t2, 0x0f000f00, 5
    or t2, t0, t1
    check t2, 0xfff0fff0, 6
    xor t2, t0, t1
    check t2, 0xf0f0f0f0, 7
    xori t2, t0, -1             # not
    check t2, 0x00ff00ff, 8
    andi t2, t0, 0x7f0
    check t2, 0x700, 9
    ori t2, zero, -1
    check t2, -1, 10

    # ---- shifts ----
    li t0, 0x80000000
    srli t1, t0, 31
    check t1, 1, 11
    srai t1, t0, 31
    check t1, -1, 12
    li t3, 1
    slli t1, t3, 31
    check t1, 0x80000000, 13
    li t2, 33                   # only the low 5 bits count: shift by 1
    sll t1, t3, t2
    check t1, 2, 14
    sra t1, t0, t2
    check t1, 0xc0000000, 15
    srl t1, t0, t2
    check t1, 0x40000000, 16

    # ---- comparisons ----
    li t0, -1
    li t1, 1
    slt t2, t0, t1              # -1 < 1 signed
    check t2, 1, 17
    sltu t2, t0, t1             # 0xffffffff < 1 unsigned
    check t2, 0, 18
    slti t2, t0, 0
    check t2, 1, 19
    sltiu t2, t1, -1            # imm sign-extends to 0xffffffff
    check t2, 1, 20
    seqz t2, zero               # sltiu t2, zero, 1
    check t2, 1, 21

    # ---- lui / auipc ----
    lui t0, 0x12345
    check t0, 0x12345000, 22
1:
    auipc t0, 0                 # address of this instruction
    la t1, 1b
    li a0, 23
    bne t0, t1, fail

    # ---- x0 ----
    addi zero, zero, 5
    mv t0, zero
    check t0, 0, 24

    exit 0

fail:
    li a7, 93
    ecall
