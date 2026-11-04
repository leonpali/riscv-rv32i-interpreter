// Runs the programs in tests/programs/bin. Rebuild them from
// tests/programs/asm with scripts/build-tests.sh after changing a source.

use std::process::Command;

use riscv_rv32i_interpreter::cpu::{Cpu, StepResult};
use riscv_rv32i_interpreter::error::ErrorType;
use riscv_rv32i_interpreter::loader::{load, load_elf};

const MEMORY_BASE: u32 = 0x8000_0000;
const MEMORY_SIZE: usize = 1 << 20;
const MAX_STEPS: usize = 1_000_000;

/// Loads `program` at the start of memory and steps until it exits or fails.
/// Returns the final result and everything the program wrote.
fn run(program: &[u8]) -> (Result<StepResult, ErrorType>, Vec<u8>) {
    let mut cpu = Cpu::with_memory_base(MEMORY_BASE, MEMORY_SIZE);
    load(&mut cpu, program, MEMORY_BASE).unwrap();
    for _ in 0..MAX_STEPS {
        match cpu.step() {
            Ok(StepResult::Continue) => {}
            result => return (result, cpu.output),
        }
    }
    panic!("program did not stop within {MAX_STEPS} steps (pc = {:#010x})", cpu.pc);
}

macro_rules! program {
    ($name:literal) => {
        include_bytes!(concat!("programs/bin/", $name, ".bin"))
    };
}

// ---------- programs run through the library ----------

#[test]
fn exit() {
    assert_eq!(run(program!("exit")), (Ok(StepResult::Exit(42)), vec![]));
}

#[test]
fn hello() {
    assert_eq!(
        run(program!("hello")),
        (Ok(StepResult::Exit(0)), b"Hello, world!\n".to_vec())
    );
}

#[test]
fn fib() {
    assert_eq!(run(program!("fib")), (Ok(StepResult::Exit(55)), vec![]));
}

// Self-checking programs exit with 0, or with the number of the failing check.

#[test]
fn alu() {
    assert_eq!(run(program!("alu")).0, Ok(StepResult::Exit(0)), "see the check number in alu.S");
}

#[test]
fn branches() {
    assert_eq!(
        run(program!("branches")).0,
        Ok(StepResult::Exit(0)),
        "see the check number in branches.S"
    );
}

#[test]
fn memory() {
    assert_eq!(
        run(program!("memory")).0,
        Ok(StepResult::Exit(0)),
        "see the check number in memory.S"
    );
}

#[test]
fn ebreak() {
    assert_eq!(run(program!("ebreak")), (Err(ErrorType::Breakpoint(0x80000004)), vec![]));
}

// ---------- the same programs as ELF files ----------

/// Like `run`, but loads the ELF version through `load_elf`.
fn run_elf(file: &[u8]) -> (Result<StepResult, ErrorType>, Vec<u8>) {
    let mut cpu = Cpu::with_memory_base(MEMORY_BASE, MEMORY_SIZE);
    load_elf(&mut cpu, file).unwrap();
    for _ in 0..MAX_STEPS {
        match cpu.step() {
            Ok(StepResult::Continue) => {}
            result => return (result, cpu.output),
        }
    }
    panic!("program did not stop within {MAX_STEPS} steps (pc = {:#010x})", cpu.pc);
}

macro_rules! elf {
    ($name:literal) => {
        include_bytes!(concat!("programs/elf/", $name, ".elf"))
    };
}

#[test]
fn elf_programs_match_flat_binaries() {
    assert_eq!(run_elf(elf!("exit")), run(program!("exit")));
    assert_eq!(run_elf(elf!("hello")), run(program!("hello")));
    assert_eq!(run_elf(elf!("fib")), run(program!("fib")));
    assert_eq!(run_elf(elf!("alu")).0, Ok(StepResult::Exit(0)));
    assert_eq!(run_elf(elf!("branches")).0, Ok(StepResult::Exit(0)));
    // code and data in one PT_LOAD segment, with the data a page after the code
    assert_eq!(run_elf(elf!("memory")).0, Ok(StepResult::Exit(0)));
    assert_eq!(run_elf(elf!("ebreak")), run(program!("ebreak")));
}

// ---------- the interpreter binary (main.rs) ----------

fn interpreter() -> Command {
    Command::new(env!("CARGO_BIN_EXE_riscv-rv32i-interpreter"))
}

fn bin_path(name: &str) -> String {
    format!("{}/tests/programs/bin/{name}.bin", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn cli_exit_code() {
    let out = interpreter().arg(bin_path("exit")).output().unwrap();
    assert_eq!(out.status.code(), Some(42));
    assert!(out.stdout.is_empty());
}

#[test]
fn cli_prints_output() {
    let out = interpreter().arg(bin_path("hello")).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"Hello, world!\n");
}

#[test]
fn cli_reports_errors() {
    let out = interpreter().arg(bin_path("ebreak")).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Breakpoint"), "stderr was: {stderr}");
    assert!(stderr.contains("0x80000004"), "stderr was: {stderr}");
}

#[test]
fn cli_runs_elf_files() {
    let path = format!("{}/tests/programs/elf/hello.elf", env!("CARGO_MANIFEST_DIR"));
    let out = interpreter().arg(path).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"Hello, world!\n");
}

#[test]
fn cli_without_arguments() {
    let out = interpreter().output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("usage"));
}

#[test]
fn cli_missing_file() {
    let out = interpreter().arg(bin_path("does-not-exist")).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}
