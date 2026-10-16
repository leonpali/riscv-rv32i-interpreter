# Helpers for self-checking test programs.
#
# A program exits with 0 if every check passes, or with the number of the
# first check that failed. Every program must define a `fail` label that
# exits with the code in a0:
#
#     fail:
#         li a7, 93
#         ecall
#
# `check` uses t6 and a0, so don't keep test values in those registers.

# Fail with exit code `num` unless `reg` == `expected`.
.macro check reg, expected, num
    li t6, \expected
    li a0, \num
    bne \reg, t6, fail
.endm

# exit(code)
.macro exit code
    li a0, \code
    li a7, 93
    ecall
.endm
