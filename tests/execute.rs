use riscv_rv32i_interpreter::cpu::{Cpu, StepResult};
use riscv_rv32i_interpreter::error::ErrorType;
use riscv_rv32i_interpreter::execute::execute;
use riscv_rv32i_interpreter::instruction::Instruction;
use riscv_rv32i_interpreter::instruction::Instruction::*;

const START_PC: u32 = 0x100;

fn cpu() -> Cpu {
    let mut cpu = Cpu::new(1024);
    cpu.pc = START_PC;
    cpu
}

fn alu(inst: Instruction, x2: u32, x3: u32) -> u32 {
    let mut cpu = cpu();
    cpu.registers[2] = x2;
    cpu.registers[3] = x3;
    execute(&mut cpu, inst).unwrap();
    assert_eq!(cpu.pc, START_PC + 4, "{inst:?} should advance pc by 4");
    cpu.registers[1]
}

fn branch(inst: Instruction, x1: u32, x2: u32) -> u32 {
    let mut cpu = cpu();
    cpu.registers[1] = x1;
    cpu.registers[2] = x2;
    execute(&mut cpu, inst).unwrap();
    cpu.pc
}

const NEG1: u32 = -1i32 as u32;
const TAKEN: u32 = START_PC + 16;
const NOT_TAKEN: u32 = START_PC + 4;


#[test]
fn lui() {
    assert_eq!(alu(Lui { rd: 1, imm: 0x12345000 }, 0, 0), 0x12345000);
    assert_eq!(alu(Lui { rd: 1, imm: -4096 }, 0, 0), 0xfffff000);
}

#[test]
fn auipc() {
    assert_eq!(alu(Auipc { rd: 1, imm: 0x1000 }, 0, 0), START_PC + 0x1000);
    assert_eq!(alu(Auipc { rd: 1, imm: -4096 }, 0, 0), 0xfffff100);
}


#[test]
fn addi() {
    assert_eq!(alu(Addi { rd: 1, rs1: 2, imm: 3 }, 5, 0), 8);
    assert_eq!(alu(Addi { rd: 1, rs1: 2, imm: -10 }, 5, 0), -5i32 as u32);
    assert_eq!(alu(Addi { rd: 1, rs1: 2, imm: 1 }, 0xffffffff, 0), 0);
}

#[test]
fn slti_is_signed() {
    assert_eq!(alu(Slti { rd: 1, rs1: 2, imm: 0 }, NEG1, 0), 1);
    assert_eq!(alu(Slti { rd: 1, rs1: 2, imm: 5 }, 5, 0), 0);
    assert_eq!(alu(Slti { rd: 1, rs1: 2, imm: -1 }, 0, 0), 0);
}

#[test]
fn sltiu_sign_extends_then_compares_unsigned() {
    assert_eq!(alu(Sltiu { rd: 1, rs1: 2, imm: -1 }, 5, 0), 1);
    assert_eq!(alu(Sltiu { rd: 1, rs1: 2, imm: 0 }, NEG1, 0), 0);
    assert_eq!(alu(Sltiu { rd: 1, rs1: 2, imm: 1 }, 0, 0), 1);
}

#[test]
fn logical_immediates() {
    assert_eq!(alu(Xori { rd: 1, rs1: 2, imm: -1 }, 0x0f0f0f0f, 0), 0xf0f0f0f0);
    assert_eq!(alu(Ori { rd: 1, rs1: 2, imm: 0x0f }, 0xf0, 0), 0xff);
    assert_eq!(alu(Ori { rd: 1, rs1: 2, imm: -2048 }, 0, 0), 0xfffff800);
    assert_eq!(alu(Andi { rd: 1, rs1: 2, imm: 0xff }, 0x12345678, 0), 0x78);
    assert_eq!(alu(Andi { rd: 1, rs1: 2, imm: -16 }, 0x12345678, 0), 0x12345670);
}

#[test]
fn shift_immediates() {
    assert_eq!(alu(Slli { rd: 1, rs1: 2, shamt: 31 }, 1, 0), 0x80000000);
    assert_eq!(alu(Srli { rd: 1, rs1: 2, shamt: 31 }, 0x80000000, 0), 1);
    assert_eq!(alu(Srai { rd: 1, rs1: 2, shamt: 31 }, 0x80000000, 0), 0xffffffff);
    assert_eq!(alu(Srai { rd: 1, rs1: 2, shamt: 4 }, 0x80000000, 0), 0xf8000000);
    assert_eq!(alu(Srai { rd: 1, rs1: 2, shamt: 4 }, 0x70000000, 0), 0x07000000);
}


#[test]
fn add_sub_wrap() {
    assert_eq!(alu(Add { rd: 1, rs1: 2, rs2: 3 }, 2, 3), 5);
    assert_eq!(alu(Add { rd: 1, rs1: 2, rs2: 3 }, 0xffffffff, 1), 0);
    assert_eq!(alu(Sub { rd: 1, rs1: 2, rs2: 3 }, 10, 3), 7);
    assert_eq!(alu(Sub { rd: 1, rs1: 2, rs2: 3 }, 0, 1), 0xffffffff);
}

#[test]
fn register_shifts_use_low_5_bits_of_rs2() {
    assert_eq!(alu(Sll { rd: 1, rs1: 2, rs2: 3 }, 1, 4), 16);
    assert_eq!(alu(Sll { rd: 1, rs1: 2, rs2: 3 }, 1, 33), 2);
    assert_eq!(alu(Srl { rd: 1, rs1: 2, rs2: 3 }, 0x80000000, 31), 1);
    assert_eq!(alu(Srl { rd: 1, rs1: 2, rs2: 3 }, 0x80000000, 32), 0x80000000);
    assert_eq!(alu(Sra { rd: 1, rs1: 2, rs2: 3 }, 0x80000000, 4), 0xf8000000);
    assert_eq!(alu(Sra { rd: 1, rs1: 2, rs2: 3 }, 0x80000000, 0xffffffe4), 0xf8000000);
}

#[test]
fn slt_signed_vs_sltu_unsigned() {
    assert_eq!(alu(Slt { rd: 1, rs1: 2, rs2: 3 }, NEG1, 1), 1);
    assert_eq!(alu(Sltu { rd: 1, rs1: 2, rs2: 3 }, NEG1, 1), 0);
    assert_eq!(alu(Slt { rd: 1, rs1: 2, rs2: 3 }, 1, NEG1), 0);
    assert_eq!(alu(Sltu { rd: 1, rs1: 2, rs2: 3 }, 1, NEG1), 1);
    assert_eq!(alu(Slt { rd: 1, rs1: 2, rs2: 3 }, 7, 7), 0);
    assert_eq!(alu(Sltu { rd: 1, rs1: 2, rs2: 3 }, 7, 7), 0);
}

#[test]
fn logical_registers() {
    assert_eq!(alu(Xor { rd: 1, rs1: 2, rs2: 3 }, 0xff00ff00, 0x0ff00ff0), 0xf0f0f0f0);
    assert_eq!(alu(Or { rd: 1, rs1: 2, rs2: 3 }, 0xff00ff00, 0x0ff00ff0), 0xfff0fff0);
    assert_eq!(alu(And { rd: 1, rs1: 2, rs2: 3 }, 0xff00ff00, 0x0ff00ff0), 0x0f000f00);
}


#[test]
fn writes_to_x0_are_ignored() {
    let mut cpu = cpu();
    execute(&mut cpu, Addi { rd: 0, rs1: 0, imm: 5 }).unwrap();
    execute(&mut cpu, Lui { rd: 0, imm: 0x12345000 }).unwrap();
    cpu.registers[2] = 7;
    execute(&mut cpu, Add { rd: 0, rs1: 2, rs2: 2 }).unwrap();
    assert_eq!(cpu.registers[0], 0);
}

#[test]
fn x0_reads_as_zero() {
    assert_eq!(alu(Add { rd: 1, rs1: 0, rs2: 3 }, 0, 9), 9);
}


#[test]
fn jal() {
    let mut cpu = cpu();
    execute(&mut cpu, Jal { rd: 1, imm: 16 }).unwrap();
    assert_eq!(cpu.registers[1], START_PC + 4);
    assert_eq!(cpu.pc, START_PC + 16);

    let mut cpu = self::cpu();
    execute(&mut cpu, Jal { rd: 1, imm: -0x100 }).unwrap();
    assert_eq!(cpu.pc, 0);
}

#[test]
fn jal_with_x0_is_plain_jump() {
    let mut cpu = cpu();
    execute(&mut cpu, Jal { rd: 0, imm: 8 }).unwrap();
    assert_eq!(cpu.registers[0], 0);
    assert_eq!(cpu.pc, START_PC + 8);
}

#[test]
fn jalr() {
    let mut cpu = cpu();
    cpu.registers[2] = 0x200;
    execute(&mut cpu, Jalr { rd: 1, rs1: 2, imm: 8 }).unwrap();
    assert_eq!(cpu.registers[1], START_PC + 4);
    assert_eq!(cpu.pc, 0x208);

    let mut cpu = self::cpu();
    cpu.registers[2] = 0x200;
    execute(&mut cpu, Jalr { rd: 1, rs1: 2, imm: -4 }).unwrap();
    assert_eq!(cpu.pc, 0x1fc);
}

#[test]
fn jalr_clears_lowest_bit_of_target() {
    let mut cpu = cpu();
    cpu.registers[2] = 0x201;
    execute(&mut cpu, Jalr { rd: 1, rs1: 2, imm: 4 }).unwrap();
    assert_eq!(cpu.pc, 0x204);
}

#[test]
fn jalr_reads_rs1_before_writing_rd() {
    // jalr x1, 0(x1): target must be the old x1, not the new return address
    let mut cpu = cpu();
    cpu.registers[1] = 0x200;
    execute(&mut cpu, Jalr { rd: 1, rs1: 1, imm: 0 }).unwrap();
    assert_eq!(cpu.pc, 0x200);
    assert_eq!(cpu.registers[1], START_PC + 4);
}


#[test]
fn beq_bne() {
    assert_eq!(branch(Beq { rs1: 1, rs2: 2, imm: 16 }, 5, 5), TAKEN);
    assert_eq!(branch(Beq { rs1: 1, rs2: 2, imm: 16 }, 5, 6), NOT_TAKEN);
    assert_eq!(branch(Bne { rs1: 1, rs2: 2, imm: 16 }, 5, 6), TAKEN);
    assert_eq!(branch(Bne { rs1: 1, rs2: 2, imm: 16 }, 5, 5), NOT_TAKEN);
}

#[test]
fn blt_bge_are_signed() {
    assert_eq!(branch(Blt { rs1: 1, rs2: 2, imm: 16 }, NEG1, 1), TAKEN);
    assert_eq!(branch(Blt { rs1: 1, rs2: 2, imm: 16 }, 1, NEG1), NOT_TAKEN);
    assert_eq!(branch(Blt { rs1: 1, rs2: 2, imm: 16 }, 1, 1), NOT_TAKEN);
    assert_eq!(branch(Bge { rs1: 1, rs2: 2, imm: 16 }, 1, NEG1), TAKEN);
    assert_eq!(branch(Bge { rs1: 1, rs2: 2, imm: 16 }, 1, 1), TAKEN);
    assert_eq!(branch(Bge { rs1: 1, rs2: 2, imm: 16 }, NEG1, 1), NOT_TAKEN);
}

#[test]
fn bltu_bgeu_are_unsigned() {
    assert_eq!(branch(Bltu { rs1: 1, rs2: 2, imm: 16 }, 1, NEG1), TAKEN);
    assert_eq!(branch(Bltu { rs1: 1, rs2: 2, imm: 16 }, NEG1, 1), NOT_TAKEN);
    assert_eq!(branch(Bltu { rs1: 1, rs2: 2, imm: 16 }, 1, 1), NOT_TAKEN);
    assert_eq!(branch(Bgeu { rs1: 1, rs2: 2, imm: 16 }, NEG1, 1), TAKEN);
    assert_eq!(branch(Bgeu { rs1: 1, rs2: 2, imm: 16 }, 1, 1), TAKEN);
    assert_eq!(branch(Bgeu { rs1: 1, rs2: 2, imm: 16 }, 1, NEG1), NOT_TAKEN);
}

#[test]
fn branch_backwards() {
    assert_eq!(branch(Beq { rs1: 1, rs2: 2, imm: -16 }, 0, 0), START_PC - 16);
}


fn cpu_with_data() -> Cpu {
    let mut cpu = cpu();
    cpu.memory.write_u32(0x200, 0xdeadbeef).unwrap();
    cpu.memory.write_u8(0x204, 0x7f).unwrap();
    cpu.registers[2] = 0x200;
    cpu
}

fn load(inst: Instruction) -> u32 {
    let mut cpu = cpu_with_data();
    execute(&mut cpu, inst).unwrap();
    assert_eq!(cpu.pc, START_PC + 4, "{inst:?} should advance pc by 4");
    cpu.registers[1]
}

#[test]
fn lw() {
    assert_eq!(load(Lw { rd: 1, rs1: 2, imm: 0 }), 0xdeadbeef);
}

#[test]
fn lb_sign_extends_lbu_zero_extends() {
    assert_eq!(load(Lb { rd: 1, rs1: 2, imm: 0 }), 0xffffffef);
    assert_eq!(load(Lbu { rd: 1, rs1: 2, imm: 0 }), 0x000000ef);
    assert_eq!(load(Lb { rd: 1, rs1: 2, imm: 4 }), 0x7f);
}

#[test]
fn lh_sign_extends_lhu_zero_extends() {
    assert_eq!(load(Lh { rd: 1, rs1: 2, imm: 0 }), 0xffffbeef);
    assert_eq!(load(Lhu { rd: 1, rs1: 2, imm: 0 }), 0x0000beef);
    assert_eq!(load(Lh { rd: 1, rs1: 2, imm: 2 }), 0xffffdead);
}

#[test]
fn load_with_negative_offset() {
    let mut cpu = cpu_with_data();
    cpu.registers[2] = 0x204;
    execute(&mut cpu, Lw { rd: 1, rs1: 2, imm: -4 }).unwrap();
    assert_eq!(cpu.registers[1], 0xdeadbeef);
}

#[test]
fn load_out_of_bounds() {
    let mut cpu = cpu();
    cpu.registers[2] = 1024;
    assert_eq!(
        execute(&mut cpu, Lw { rd: 1, rs1: 2, imm: 0 }),
        Err(ErrorType::MemoryOutOfBounds(1024))
    );

    cpu.registers[2] = 1022;
    assert_eq!(
        execute(&mut cpu, Lw { rd: 1, rs1: 2, imm: 0 }),
        Err(ErrorType::MemoryOutOfBounds(1022))
    );
}

#[test]
fn load_address_wraps_around() {
    // 0 + (-4) is address 0xfffffffc, not an overflow panic
    let mut cpu = cpu();
    assert_eq!(
        execute(&mut cpu, Lw { rd: 1, rs1: 0, imm: -4 }),
        Err(ErrorType::MemoryOutOfBounds(0xfffffffc))
    );
}

fn store(inst: Instruction) -> u32 {
    let mut cpu = cpu();
    cpu.memory.write_u32(0x200, 0xffffffff).unwrap();
    cpu.registers[2] = 0x200;
    cpu.registers[3] = 0x12345678;
    execute(&mut cpu, inst).unwrap();
    assert_eq!(cpu.pc, START_PC + 4, "{inst:?} should advance pc by 4");
    cpu.memory.read_u32(0x200).unwrap()
}

#[test]
fn sw() {
    assert_eq!(store(Sw { rs1: 2, rs2: 3, imm: 0 }), 0x12345678);
}

#[test]
fn sh_writes_low_half_only() {
    assert_eq!(store(Sh { rs1: 2, rs2: 3, imm: 0 }), 0xffff5678);
}

#[test]
fn sb_writes_low_byte_only() {
    assert_eq!(store(Sb { rs1: 2, rs2: 3, imm: 0 }), 0xffffff78);
}

#[test]
fn store_with_negative_offset() {
    let mut cpu = cpu();
    cpu.registers[2] = 0x204;
    cpu.registers[3] = 0xcafebabe;
    execute(&mut cpu, Sw { rs1: 2, rs2: 3, imm: -4 }).unwrap();
    assert_eq!(cpu.memory.read_u32(0x200), Ok(0xcafebabe));
}

#[test]
fn store_out_of_bounds() {
    let mut cpu = cpu();
    cpu.registers[2] = 1024;
    assert_eq!(
        execute(&mut cpu, Sw { rs1: 2, rs2: 3, imm: 0 }),
        Err(ErrorType::MemoryOutOfBounds(1024))
    );
}


#[test]
fn fence_is_a_no_op() {
    let mut cpu = cpu();
    cpu.registers[5] = 42;
    execute(&mut cpu, Fence).unwrap();
    assert_eq!(cpu.pc, START_PC + 4);
    assert_eq!(cpu.registers[5], 42);
}

#[test]
fn non_system_instructions_continue() {
    let mut cpu = cpu();
    assert_eq!(execute(&mut cpu, Addi { rd: 1, rs1: 0, imm: 1 }), Ok(StepResult::Continue));
    assert_eq!(execute(&mut cpu, Fence), Ok(StepResult::Continue));
}

#[test]
fn ecall_exit() {
    let mut cpu = cpu();
    cpu.registers[17] = 93;
    cpu.registers[10] = 42;
    assert_eq!(execute(&mut cpu, Ecall), Ok(StepResult::Exit(42)));
}

#[test]
fn ecall_unsupported_syscall() {
    let mut cpu = cpu();
    cpu.registers[17] = 9999;
    assert_eq!(execute(&mut cpu, Ecall), Err(ErrorType::UnsupportedSyscall(9999)));
}

#[test]
fn ebreak() {
    let mut cpu = cpu();
    assert_eq!(execute(&mut cpu, Ebreak), Err(ErrorType::Breakpoint(START_PC)));
}
