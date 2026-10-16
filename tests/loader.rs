use riscv_rv32i_interpreter::cpu::{Cpu, StepResult};
use riscv_rv32i_interpreter::error::ErrorType;
use riscv_rv32i_interpreter::loader::load;

/// Turns instruction words into the bytes of a flat binary (little-endian).
fn to_bytes(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|w| w.to_le_bytes()).collect()
}

#[test]
fn copies_program_to_base() {
    let mut cpu = Cpu::new(1024);
    load(&mut cpu, &[1, 2, 3, 4, 5], 0x100).unwrap();
    for (i, expected) in [1, 2, 3, 4, 5].into_iter().enumerate() {
        assert_eq!(cpu.memory.read_u8(0x100 + i as u32), Ok(expected));
    }
    // memory around the program is untouched
    assert_eq!(cpu.memory.read_u8(0xff), Ok(0));
    assert_eq!(cpu.memory.read_u8(0x105), Ok(0));
}

#[test]
fn sets_pc_to_base() {
    let mut cpu = Cpu::new(1024);
    load(&mut cpu, &[0; 8], 0x100).unwrap();
    assert_eq!(cpu.pc, 0x100);

    let mut cpu = Cpu::new(1024);
    cpu.pc = 0x40;
    load(&mut cpu, &[0; 8], 0).unwrap();
    assert_eq!(cpu.pc, 0);
}

#[test]
fn bytes_are_stored_little_endian() {
    let mut cpu = Cpu::new(1024);
    load(&mut cpu, &to_bytes(&[0x00500093]), 0).unwrap();
    assert_eq!(cpu.memory.read_u8(0), Ok(0x93));
    assert_eq!(cpu.memory.read_u32(0), Ok(0x00500093));
}

#[test]
fn empty_program() {
    let mut cpu = Cpu::new(1024);
    assert_eq!(load(&mut cpu, &[], 0x100), Ok(()));
    assert_eq!(cpu.pc, 0x100);
}

#[test]
fn program_filling_all_of_memory() {
    let mut cpu = Cpu::new(1024);
    let program = vec![0xab; 1024];
    assert_eq!(load(&mut cpu, &program, 0), Ok(()));
    assert_eq!(cpu.memory.read_u8(1023), Ok(0xab));
}

#[test]
fn program_too_large() {
    let mut cpu = Cpu::new(1024);
    cpu.pc = 0x40;
    let program = vec![0; 1025];
    assert!(matches!(load(&mut cpu, &program, 0), Err(ErrorType::MemoryOutOfBounds(_))));
    // a failed load leaves pc alone
    assert_eq!(cpu.pc, 0x40);
}

#[test]
fn program_past_end_of_memory() {
    let mut cpu = Cpu::new(1024);
    assert!(matches!(
        load(&mut cpu, &[0; 8], 1020),
        Err(ErrorType::MemoryOutOfBounds(_))
    ));
}

#[test]
fn misaligned_base() {
    // the entry point is a jump target, so it must be a multiple of 4
    let mut cpu = Cpu::new(1024);
    assert_eq!(
        load(&mut cpu, &[0; 8], 2),
        Err(ErrorType::InstructionAddressMisaligned(2))
    );
}

#[test]
fn loaded_program_runs() {
    let program = to_bytes(&[
        0x02a00513, // addi a0, x0, 42
        0x05d00893, // addi a7, x0, 93  (exit)
        0x00000073, // ecall
    ]);
    let mut cpu = Cpu::new(1024);
    load(&mut cpu, &program, 0x100).unwrap();
    assert_eq!(cpu.step(), Ok(StepResult::Continue));
    assert_eq!(cpu.step(), Ok(StepResult::Continue));
    assert_eq!(cpu.step(), Ok(StepResult::Exit(42)));
}
