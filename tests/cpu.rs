use riscv_rv32i_interpreter::cpu::{Cpu, StepResult};
use riscv_rv32i_interpreter::error::ErrorType;


fn cpu_with_program(program: &[u32]) -> Cpu {
    let mut cpu = Cpu::new(1024);
    for (i, &word) in program.iter().enumerate() {
        cpu.memory.write_u32(i as u32 * 4, word).unwrap();
    }
    cpu
}

#[test]
fn new_cpu_starts_zeroed() {
    let cpu = Cpu::new(1024);
    assert_eq!(cpu.registers, [0; 32]);
    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.memory.read_u32(1020), Ok(0));
    assert_eq!(cpu.memory.read_u32(1024), Err(ErrorType::MemoryOutOfBounds(1024)));
}

#[test]
fn step_executes_one_instruction() {
    let mut cpu = cpu_with_program(&[
        0x00500093, // addi x1, x0, 5
    ]);
    assert_eq!(cpu.step(), Ok(StepResult::Continue));
    assert_eq!(cpu.registers[1], 5);
    assert_eq!(cpu.pc, 4);
}

#[test]
fn step_runs_straight_line_program() {
    let mut cpu = cpu_with_program(&[
        0x00500093, // addi x1, x0, 5
        0x00700113, // addi x2, x0, 7
        0x002081b3, // add  x3, x1, x2
    ]);
    for _ in 0..3 {
        assert_eq!(cpu.step(), Ok(StepResult::Continue));
    }
    assert_eq!(cpu.registers[3], 12);
    assert_eq!(cpu.pc, 12);
}

#[test]
fn step_runs_loop() {
    let mut cpu = cpu_with_program(&[
        0x00300093, //       addi x1, x0, 3
        0x00000113, //       addi x2, x0, 0
        0x00210113, // loop: addi x2, x2, 2
        0xfff08093, //       addi x1, x1, -1
        0xfe009ce3, //       bne  x1, x0, loop
    ]);
    for _ in 0..11 {
        assert_eq!(cpu.step(), Ok(StepResult::Continue));
    }
    assert_eq!(cpu.registers[1], 0);
    assert_eq!(cpu.registers[2], 6);
    assert_eq!(cpu.pc, 20);
}

#[test]
fn step_jumps() {
    let mut cpu = cpu_with_program(&[
        0x00c000ef, // jal x1, +12
    ]);
    assert_eq!(cpu.step(), Ok(StepResult::Continue));
    assert_eq!(cpu.registers[1], 4);
    assert_eq!(cpu.pc, 12);
}

#[test]
fn step_exit_syscall() {
    let mut cpu = cpu_with_program(&[
        0x02a00513, // addi a0, x0, 42
        0x05d00893, // addi a7, x0, 93  (exit)
        0x00000073, // ecall
    ]);
    assert_eq!(cpu.step(), Ok(StepResult::Continue));
    assert_eq!(cpu.step(), Ok(StepResult::Continue));
    assert_eq!(cpu.step(), Ok(StepResult::Exit(42)));
}

#[test]
fn step_illegal_instruction() {
    let mut cpu = Cpu::new(1024);
    assert_eq!(cpu.step(), Err(ErrorType::IllegalInstruction(0)));
    assert_eq!(cpu.pc, 0);
}

#[test]
fn step_with_pc_out_of_bounds() {
    let mut cpu = Cpu::new(1024);
    cpu.pc = 1024;
    assert_eq!(cpu.step(), Err(ErrorType::MemoryOutOfBounds(1024)));
}
