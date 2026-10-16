use riscv_rv32i_interpreter::cpu::{Cpu, StepResult};
use riscv_rv32i_interpreter::error::ErrorType;
use riscv_rv32i_interpreter::syscall::handle;

// Linux RISC-V syscall convention: number in a7, arguments in a0..a2,
// return value in a0.
const A0: usize = 10;
const A1: usize = 11;
const A2: usize = 12;
const A7: usize = 17;

const SYS_WRITE: u32 = 64;
const SYS_EXIT: u32 = 93;
const STDOUT: u32 = 1;
const EBADF: u32 = -9i32 as u32;

const START_PC: u32 = 0x100;

fn cpu() -> Cpu {
    let mut cpu = Cpu::new(1024);
    cpu.pc = START_PC;
    cpu
}

/// CPU set up for write(fd, buf = 0x200, len) with `data` stored at 0x200.
fn cpu_for_write(fd: u32, data: &[u8], len: u32) -> Cpu {
    let mut cpu = cpu();
    for (i, &byte) in data.iter().enumerate() {
        cpu.memory.write_u8(0x200 + i as u32, byte).unwrap();
    }
    cpu.registers[A7] = SYS_WRITE;
    cpu.registers[A0] = fd;
    cpu.registers[A1] = 0x200;
    cpu.registers[A2] = len;
    cpu
}

// ---------- exit ----------

#[test]
fn exit_returns_a0() {
    let mut cpu = cpu();
    cpu.registers[A7] = SYS_EXIT;
    cpu.registers[A0] = 42;
    assert_eq!(handle(&mut cpu), Ok(StepResult::Exit(42)));
}

#[test]
fn exit_with_zero() {
    let mut cpu = cpu();
    cpu.registers[A7] = SYS_EXIT;
    assert_eq!(handle(&mut cpu), Ok(StepResult::Exit(0)));
}

#[test]
fn exit_with_negative_code() {
    // exit(-1) arrives as the raw register value
    let mut cpu = cpu();
    cpu.registers[A7] = SYS_EXIT;
    cpu.registers[A0] = -1i32 as u32;
    assert_eq!(handle(&mut cpu), Ok(StepResult::Exit(0xffffffff)));
}

// ---------- write ----------

#[test]
fn write_to_stdout() {
    let mut cpu = cpu_for_write(STDOUT, b"hello\n", 6);
    assert_eq!(handle(&mut cpu), Ok(StepResult::Continue));
    assert_eq!(cpu.output, b"hello\n");
    // returns the number of bytes written
    assert_eq!(cpu.registers[A0], 6);
}

#[test]
fn write_only_len_bytes() {
    let mut cpu = cpu_for_write(STDOUT, b"hello\n", 4);
    handle(&mut cpu).unwrap();
    assert_eq!(cpu.output, b"hell");
    assert_eq!(cpu.registers[A0], 4);
}

#[test]
fn write_appends_to_output() {
    let mut cpu = cpu_for_write(STDOUT, b"ab", 2);
    handle(&mut cpu).unwrap();
    // a0 now holds the return value, so set the fd again
    cpu.registers[A0] = STDOUT;
    handle(&mut cpu).unwrap();
    assert_eq!(cpu.output, b"abab");
}

#[test]
fn write_zero_bytes() {
    let mut cpu = cpu_for_write(STDOUT, b"hello", 0);
    assert_eq!(handle(&mut cpu), Ok(StepResult::Continue));
    assert!(cpu.output.is_empty());
    assert_eq!(cpu.registers[A0], 0);
}

#[test]
fn write_does_not_touch_pc() {
    // advancing pc past the ecall is execute's job
    let mut cpu = cpu_for_write(STDOUT, b"x", 1);
    handle(&mut cpu).unwrap();
    assert_eq!(cpu.pc, START_PC);
}

#[test]
fn write_to_unsupported_fd_returns_ebadf() {
    // like Linux: not a crash, just -EBADF in a0
    let mut cpu = cpu_for_write(7, b"hello", 5);
    assert_eq!(handle(&mut cpu), Ok(StepResult::Continue));
    assert!(cpu.output.is_empty());
    assert_eq!(cpu.registers[A0], EBADF);
}

#[test]
fn write_buffer_out_of_bounds() {
    // buffer starts inside memory but runs past the end (memory is 1024 bytes)
    let mut cpu = cpu_for_write(STDOUT, b"", 0);
    cpu.registers[A1] = 1020;
    cpu.registers[A2] = 8;
    assert!(matches!(handle(&mut cpu), Err(ErrorType::MemoryOutOfBounds(_))));
    // no partial output
    assert!(cpu.output.is_empty());
}

// ---------- unsupported ----------

#[test]
fn unsupported_syscall() {
    let mut cpu = cpu();
    cpu.registers[A7] = 9999;
    assert_eq!(handle(&mut cpu), Err(ErrorType::UnsupportedSyscall(9999)));
}
