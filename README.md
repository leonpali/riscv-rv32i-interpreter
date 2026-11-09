# riscv-rv32i-interpreter

An interpreter for the 32-bit RISC-V base integer instruction set (**RV32I**),
written in Rust with no dependencies. It loads a RISC-V program (ELF or flat
binary), executes it instruction by instruction, and supports enough of a
system interface for programs to print output and exit with a status code.

It passes all 42 tests of the official RISC-V ISA test suite for RV32I
(`rv32ui-p-*` from [riscv-tests](https://github.com/riscv-software-src/riscv-tests)),
running the test binaries unmodified.

## Features

- **All 40 RV32I instructions**: arithmetic, logic, shifts, comparisons,
  loads and stores, branches, jumps, `LUI`/`AUIPC`, `FENCE`, `ECALL`, `EBREAK`
- **Zicsr**: the six CSR instructions (`CSRRW`, `CSRRS`, `CSRRC` and their
  immediate forms), with storage for all 4096 CSRs
- **Zifencei**: `FENCE.I`
- **`MRET`**, enough for the machine-mode startup code of the official tests
- **ELF loader** for 32-bit little-endian RISC-V executables, plus a loader for
  flat binaries
- **System calls** via `ECALL`, following the Linux RISC-V convention:
  `exit` (93) and `write` (64) to stdout
- **Precise errors**: illegal instructions, out-of-bounds memory accesses,
  misaligned jump targets and breakpoints stop execution with the address
  where they happened, without changing any state

## Running a program

Requires a Rust toolchain (edition 2024).

```sh
cargo build --release
./target/release/riscv-rv32i-interpreter program.elf
```

The interpreter accepts either an ELF executable or a flat binary; it detects
the format from the file's first bytes. Flat binaries are loaded at
`0x80000000` and start executing at their first byte.

Whatever the program writes to stdout is printed, and the interpreter exits
with the program's exit code:

```sh
$ ./target/release/riscv-rv32i-interpreter tests/programs/elf/hello.elf
Hello, world!
$ echo $?
0
```

| Exit code | Meaning |
|---|---|
| program's code | the program called `exit` |
| 1 | the program stopped with an error, printed to stderr with its `pc` (e.g. `error at pc 0x80000004: Breakpoint(2147483652)`) |
| 2 | no file given, the file can't be read, or it can't be loaded |

### Writing programs for it

Programs run on a bare machine with 1 MiB of RAM at `0x80000000`. There is no
operating system and no C library; the only services are these system calls,
made with `ECALL` (number in `a7`, arguments in `a0`–`a2`, result in `a0`):

| `a7` | Call | Arguments | Result |
|---|---|---|---|
| 93 | `exit` | `a0` = exit code | stops the program |
| 64 | `write` | `a0` = fd, `a1` = buffer address, `a2` = length | `a0` = bytes written; only fd 1 (stdout) is supported, other fds return `-EBADF` (-9) |

A minimal program:

```asm
.globl _start
_start:
    li a0, 42       # exit code
    li a7, 93       # exit
    ecall
```

Assemble and link it for `0x80000000` with a RISC-V GNU toolchain:

```sh
riscv64-unknown-elf-as -march=rv32i -mabi=ilp32 prog.S -o prog.o
riscv64-unknown-elf-ld -m elf32lriscv --no-relax --nmagic -Ttext=0x80000000 -e _start prog.o -o prog.elf
```

- `--no-relax`: otherwise the linker rewrites `la` into addressing relative to
  `gp`, which nothing sets up.
- `--nmagic`: otherwise the first segment also contains the ELF headers and
  starts one page below `0x80000000`, outside of memory.

The loader does not set up a stack, so programs that use one must set `sp`
themselves (e.g. `li sp, 0x80010000`).

## Design

The interpreter is a library (`src/lib.rs`) with a small command-line program
on top (`src/main.rs`). Each module has one job:

| Module | Responsibility |
|---|---|
| [`memory.rs`](src/memory.rs) | Byte-addressed little-endian RAM covering `base .. base + size`. All accesses are bounds-checked in one place. |
| [`instruction.rs`](src/instruction.rs) | The `Instruction` enum: one variant per instruction, with its operands as named fields. |
| [`decode.rs`](src/decode.rs) | Turns a 32-bit instruction word into an `Instruction`, or `IllegalInstruction`. |
| [`execute.rs`](src/execute.rs) | Executes one `Instruction`: updates registers, memory, CSRs and `pc`. |
| [`cpu.rs`](src/cpu.rs) | CPU state (`x0`–`x31`, `pc`, CSRs, memory, program output) and `step()`: fetch, decode, execute. |
| [`syscall.rs`](src/syscall.rs) | System calls made with `ECALL`. |
| [`loader.rs`](src/loader.rs) | Loads flat binaries and ELF executables into memory and sets `pc`. |
| [`error.rs`](src/error.rs) | `ErrorType`: everything that can stop execution or loading. |

Some decisions behind it:

- **Decode and execute are separate steps.** The decoder does all bit
  manipulation once, including sign-extending and reassembling the scrambled
  immediates of the S, B and J formats. `execute` only ever sees typed
  `Instruction` values, so each instruction is a few readable lines, and the
  decoder can be tested on its own.
- **`execute` has no catch-all arm**, so the compiler checks that every
  instruction is handled.
- **Errors leave the CPU unchanged.** `execute` computes the next `pc` and
  only commits it at the end; every fallible step (memory access, jump target
  check) uses `?` before anything is written. A failing instruction leaves
  `pc` pointing at itself, like a precise trap on real hardware. The loaders
  follow the same rule: a program that doesn't fit is rejected before any byte
  is written.
- **Guest arithmetic always wraps** (`wrapping_add`, `wrapping_sub`), matching
  RISC-V's "overflow is ignored", and shift amounts are masked to 5 bits, so
  no guest program can make the interpreter panic.
- **One bounds check for memory.** An address is turned into an index with
  `addr.wrapping_sub(base)`, so an address below `base` becomes a huge index
  and fails the same size check as one past the end. `Memory::with_base`
  refuses configurations that would extend past `0xffffffff`, which keeps
  that trick sound.
- **`x0` is handled in one place**: `write_reg` ignores writes to register 0.
- **Misaligned loads and stores are supported**, which the spec allows; a
  jump or taken branch to an address that isn't a multiple of 4 is an error,
  as the spec requires without the compressed extension.

### Simplifications

These are enough for user-level programs and the `rv32ui` tests, but not a
full privileged architecture:

- CSRs are plain storage: every CSR number is accepted, and read-only CSRs
  such as `mhartid` accept writes.
- There are no privilege modes and no traps. `MRET` jumps to `mepc` without
  updating `mstatus`, and `ECALL` is handled directly as a system call rather
  than trapping to `mtvec`.
- Program output is collected and printed when the program stops.
- There is no step limit: a program that loops forever runs forever.

## Testing

```sh
cargo test
```

The tests live in `tests/` and use only the library's public API. They come in
three layers.

### 1. Unit tests per module

| File | Tests | What they cover |
|---|---|---|
| `tests/memory.rs` | 44 | every access size, little-endian byte order, misaligned access, bounds at both ends of memory, memory at `0x80000000` and at the top of the address space, partial out-of-bounds accesses writing nothing |
| `tests/decode.rs` | 15 | every instruction, the largest and smallest branch/jump offsets, Zicsr and `MRET`, and 15 encodings that must be rejected (RV64-only, M extension, invalid `funct3`/`funct7`, `sret`, `wfi`, …) |
| `tests/execute.rs` | 50 | every instruction, with emphasis on edge cases: signed vs unsigned comparisons, overflow wrapping, shift amounts above 31, sign vs zero extension in loads, `JALR` with `rd = rs1`, writes to `x0`, CSR set/clear semantics |
| `tests/cpu.rs` | 12 | `step()` running small programs: loops, jumps, system calls, illegal instructions, `pc` out of bounds |
| `tests/syscall.rs` | 11 | `exit` and `write`, including bad file descriptors and buffers outside memory |
| `tests/loader.rs` | 29 | flat binaries and ELF files: segments, `.bss` zero-filling, physical addresses, every rejected header, segments outside memory |

The expected instruction encodings in these tests were generated with the GNU
assembler rather than encoded by hand.

### 2. Assembly test programs

`tests/programs/asm/` contains small programs that exercise the whole
pipeline (loading, decoding, executing, system calls):

| Program | Expected result |
|---|---|
| `exit.S` | exit code 42 |
| `hello.S` | prints `Hello, world!`, exit code 0 |
| `fib.S` | exit code 55: recursive `fib(10)` using calls and the stack |
| `alu.S` | 24 self-checks of arithmetic, logic, shifts and comparisons |
| `branches.S` | 17 self-checks of every branch, loops and indirect calls |
| `memory.S` | 13 self-checks of loads, stores and the stack |
| `ebreak.S` | stops with a breakpoint error |

The self-checking programs exit with 0 if every check passes, or with the
number of the first failing check. `tests/programs.rs` (14 tests) runs every
program as a flat binary and as an ELF file, and also runs the compiled
interpreter binary to check its exit codes, stdout and error messages.

The built programs are committed in `tests/programs/bin/` and
`tests/programs/elf/`, so `cargo test` doesn't need a RISC-V toolchain. After
changing a source file, rebuild them with:

```sh
scripts/build-tests.sh
```

### 3. The official RISC-V ISA tests

`tests/rv32ui.rs` runs the 42 `rv32ui-p-*` tests from
[riscv-tests](https://github.com/riscv-software-src/riscv-tests), the suite
RISC-V simulators and hardware designs are checked against. They run
unmodified: the interpreter executes the suite's own machine-mode startup code
(CSR setup and `MRET`) before each test. Each test reports its result through
the `exit` system call: 0 for a pass, or `(case << 1) | 1` for the first
failing case, which the runner turns into a message such as
`rv32ui-p-sub: case 6 failed`.

The binaries in `tests/rv32ui/` were built from riscv-tests commit `3df703c`
with `riscv64-unknown-elf-gcc` 13.2.0:

```sh
git clone --recursive https://github.com/riscv-software-src/riscv-tests.git
cd riscv-tests/isa
make XLEN=32 RISCV_PREFIX=riscv64-unknown-elf- \
    $(for t in rv32ui/*.S; do echo rv32ui-p-$(basename $t .S); done)
```

They are distributed under the riscv-tests license, included as
[`tests/rv32ui/LICENSE`](tests/rv32ui/LICENSE).

### Results

All 217 tests pass:

| Suite | Passed |
|---|---|
| Unit tests (`memory`, `decode`, `execute`, `cpu`, `syscall`, `loader`) | 161 / 161 |
| Assembly programs (`programs`) | 14 / 14 |
| Official RISC-V tests (`rv32ui-p-*`) | **42 / 42** |

The official tests cover every RV32I instruction, plus `simple`, `fence_i`
(self-modifying code), `ma_data` (misaligned loads and stores), `ld_st` and
`st_ld`. The runner was checked against deliberately broken builds: a wrong
`SUB` fails exactly `rv32ui-p-sub` at the right case number.
