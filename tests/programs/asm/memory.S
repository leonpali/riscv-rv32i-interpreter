# Loads (sign/zero extension), stores (byte/half/word) and the stack.
# Expected: exit code 0, no output. Any other code is the failing check.

.include "check.inc"

.data
value:
    .word 0xdeadbeef            # bytes in memory: ef be ad de
buf:
    .space 16

.text
.globl _start
_start:
    # ---- loads ----
    la s0, value
    lw t0, 0(s0)
    check t0, 0xdeadbeef, 1
    lb t0, 0(s0)                # 0xef, sign-extended
    check t0, 0xffffffef, 2
    lbu t0, 0(s0)
    check t0, 0xef, 3
    lh t0, 0(s0)                # 0xbeef, sign-extended
    check t0, 0xffffbeef, 4
    lhu t0, 0(s0)
    check t0, 0xbeef, 5
    lh t0, 2(s0)
    check t0, 0xffffdead, 6
    lb t0, 3(s0)
    check t0, 0xffffffde, 7

    # ---- stores ----
    la s1, buf
    li t1, 0x12345678
    sw t1, 0(s1)
    lw t0, 0(s1)
    check t0, 0x12345678, 8
    li t2, 0xab
    sb t2, 1(s1)                # replaces byte 1 only
    lw t0, 0(s1)
    check t0, 0x1234ab78, 9
    li t2, 0xcdef
    sh t2, 2(s1)                # replaces bytes 2-3 only
    lw t0, 0(s1)
    check t0, 0xcdefab78, 10

    # negative offset
    addi s2, s1, 8
    sw t1, -4(s2)
    lw t0, 4(s1)
    check t0, 0x12345678, 11

    # sb stores only the low byte
    li t2, 0xffffff01
    sb t2, 8(s1)
    lw t0, 8(s1)
    check t0, 0x00000001, 12

    # ---- stack push / pop ----
    li sp, 0x10000
    addi sp, sp, -8
    sw t1, 4(sp)
    lw t3, 4(sp)
    addi sp, sp, 8
    check t3, 0x12345678, 13

    exit 0

fail:
    li a7, 93
    ecall
