// Runs the official RISC-V ISA tests (riscv-tests, rv32ui-p-*) from
// tests/rv32ui, unmodified. Each test ends with an exit syscall:
// a0 = 0 if it passed, or (case << 1) | 1 for the first failing case.
//
// Built from https://github.com/riscv-software-src/riscv-tests with
//   make XLEN=32 RISCV_PREFIX=riscv64-unknown-elf- rv32ui-p-<name> ...

use riscv_rv32i_interpreter::cpu::{Cpu, StepResult};
use riscv_rv32i_interpreter::loader::load_elf;

const MEMORY_BASE: u32 = 0x8000_0000;
const MEMORY_SIZE: usize = 1 << 20;
const MAX_STEPS: usize = 1_000_000;

fn run(name: &str, file: &[u8]) {
    let mut cpu = Cpu::with_memory_base(MEMORY_BASE, MEMORY_SIZE);
    load_elf(&mut cpu, file).unwrap();
    for _ in 0..MAX_STEPS {
        match cpu.step() {
            Ok(StepResult::Continue) => {}
            Ok(StepResult::Exit(0)) => return,
            Ok(StepResult::Exit(code)) => panic!(
                "{name}: case {} failed (see the {name}.dump listing from riscv-tests)",
                code >> 1
            ),
            Err(e) => panic!("{name}: {e:?} at pc {:#010x}", cpu.pc),
        }
    }
    panic!("{name}: did not finish within {MAX_STEPS} steps (pc = {:#010x})", cpu.pc);
}

macro_rules! rv32ui {
    ($($test:ident),* $(,)?) => {
        $(
            #[test]
            fn $test() {
                let name = concat!("rv32ui-p-", stringify!($test));
                run(name, include_bytes!(concat!("rv32ui/rv32ui-p-", stringify!($test))));
            }
        )*
    };
}

rv32ui! {
    simple,
    add, addi,
    and, andi,
    auipc,
    beq, bge, bgeu, blt, bltu, bne,
    fence_i,
    jal, jalr,
    lb, lbu, lh, lhu, lw, ld_st,
    lui,
    ma_data,
    or, ori,
    sb, sh, sw, st_ld,
    sll, slli,
    slt, slti, sltiu, sltu,
    sra, srai,
    srl, srli,
    sub,
    xor, xori,
}
