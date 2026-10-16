use crate::cpu::{Cpu, StepResult};
use crate::error::ErrorType;
use crate::instruction::Instruction::*;
use crate::instruction::Instruction;
use crate::syscall;

pub fn execute(cpu: &mut Cpu, inst: Instruction) -> Result<StepResult, ErrorType> {
    let pc = cpu.pc;
    let mut next_pc = pc.wrapping_add(4);

    match inst {
        Addi { rd, rs1, imm } => cpu.write_reg(rd, cpu.read_reg(rs1).wrapping_add(imm as u32)),
        Slti { rd, rs1, imm } => {
            let result = (cpu.read_reg(rs1) as i32) < imm;
            cpu.write_reg(rd, result as u32);
        },
        Sltiu { rd, rs1, imm } => {
            let result = cpu.read_reg(rs1) < imm as u32;
            cpu.write_reg(rd, result as u32);
        },
        Xori { rd, rs1, imm } => cpu.write_reg(rd, cpu.read_reg(rs1) ^ (imm as u32)),
        Ori { rd, rs1, imm } => cpu.write_reg(rd, cpu.read_reg(rs1) | (imm as u32)),
        Andi { rd, rs1, imm } => cpu.write_reg(rd, cpu.read_reg(rs1) & (imm as u32)),
        Slli { rd, rs1, shamt } => cpu.write_reg(rd, cpu.read_reg(rs1).wrapping_shl(shamt as u32)),
        Srli { rd, rs1, shamt } => cpu.write_reg(rd, cpu.read_reg(rs1) >> (shamt as u32)),
        Srai { rd, rs1, shamt } => cpu.write_reg(rd, ((cpu.read_reg(rs1) as i32) >> shamt) as u32),
        Lui { rd, imm } => cpu.write_reg(rd, imm as u32),
        Auipc { rd, imm } => cpu.write_reg(rd, pc.wrapping_add(imm as u32)),
        Add { rd, rs1, rs2 } => cpu.write_reg(rd, cpu.read_reg(rs1).wrapping_add(cpu.read_reg(rs2))),
        Sub { rd, rs1, rs2 } => cpu.write_reg(rd, cpu.read_reg(rs1).wrapping_sub(cpu.read_reg(rs2))),
        Slt { rd, rs1, rs2 } => {
            let result = (cpu.read_reg(rs1) as i32) < (cpu.read_reg(rs2) as i32);
            cpu.write_reg(rd, result as u32);
        },
        Sltu { rd, rs1, rs2 } => {
            let result = cpu.read_reg(rs1) < cpu.read_reg(rs2);
            cpu.write_reg(rd, result as u32);
        },
        And { rd, rs1, rs2 } => cpu.write_reg(rd, cpu.read_reg(rs1) & cpu.read_reg(rs2)),
        Or { rd, rs1, rs2 } => cpu.write_reg(rd, cpu.read_reg(rs1) | cpu.read_reg(rs2)),
        Xor { rd, rs1, rs2 } => cpu.write_reg(rd, cpu.read_reg(rs1) ^ cpu.read_reg(rs2)),
        Sll { rd, rs1, rs2 } => {
            let shamt = cpu.read_reg(rs2) & 0x1f;
            cpu.write_reg(rd, cpu.read_reg(rs1) << shamt);
        },
        Srl { rd, rs1, rs2 } => {
            let shamt = cpu.read_reg(rs2) & 0x1f;
            cpu.write_reg(rd, cpu.read_reg(rs1) >> shamt);
        },
        Sra { rd, rs1, rs2 } => {
            let shamt = cpu.read_reg(rs2) & 0x1f;
            cpu.write_reg(rd, ((cpu.read_reg(rs1) as i32) >> shamt) as u32);
        },
        Jal { rd, imm } => {
            let target = jump_target(pc.wrapping_add(imm as u32))?;
            cpu.write_reg(rd, pc.wrapping_add(4));
            next_pc = target;
        },
        Jalr { rd, rs1, imm } => {
            let target = jump_target(cpu.read_reg(rs1).wrapping_add(imm as u32) & !1)?;
            cpu.write_reg(rd, pc.wrapping_add(4));
            next_pc = target;
        },
        Beq { rs1, rs2, imm } => {
            if cpu.read_reg(rs1) == cpu.read_reg(rs2) {
                next_pc = jump_target(pc.wrapping_add(imm as u32))?;
            }
        },
        Bne { rs1, rs2, imm } => {
            if cpu.read_reg(rs1) != cpu.read_reg(rs2) {
                next_pc = jump_target(pc.wrapping_add(imm as u32))?;
            }
        },
        Blt { rs1, rs2, imm } => {
            if (cpu.read_reg(rs1) as i32) < (cpu.read_reg(rs2) as i32) {
                next_pc = jump_target(pc.wrapping_add(imm as u32))?;
            }
        },
        Bltu { rs1, rs2, imm } => {
            if cpu.read_reg(rs1) < cpu.read_reg(rs2) {
                next_pc = jump_target(pc.wrapping_add(imm as u32))?;
            }
        },
        Bge { rs1, rs2, imm } => {
            if (cpu.read_reg(rs1) as i32) >= (cpu.read_reg(rs2) as i32) {
                next_pc = jump_target(pc.wrapping_add(imm as u32))?;
            }
        },
        Bgeu { rs1, rs2, imm } => {
            if cpu.read_reg(rs1) >= cpu.read_reg(rs2) {
                next_pc = jump_target(pc.wrapping_add(imm as u32))?;
            }
        },
        Lw { rd, rs1, imm } => {
            let addr = cpu.read_reg(rs1).wrapping_add(imm as u32);
            cpu.write_reg(rd, cpu.memory.read_u32(addr)?);
        }
        Lh { rd, rs1, imm } => {
            let addr = cpu.read_reg(rs1).wrapping_add(imm as u32);
            let val = cpu.memory.read_u16(addr)? as i16 as i32 as u32;
            cpu.write_reg(rd, val);
        },
        Lhu { rd, rs1, imm } => {
            let addr = cpu.read_reg(rs1).wrapping_add(imm as u32);
            let val = cpu.memory.read_u16(addr)? as u32;
            cpu.write_reg(rd, val);
        },
        Lb { rd, rs1, imm } => {
            let addr = cpu.read_reg(rs1).wrapping_add(imm as u32);
            let val = cpu.memory.read_u8(addr)? as i8 as i16 as i32 as u32;
            cpu.write_reg(rd, val);
        },
        Lbu { rd, rs1, imm } => {
            let addr = cpu.read_reg(rs1).wrapping_add(imm as u32);
            let val = cpu.memory.read_u8(addr)? as u32;
            cpu.write_reg(rd, val);
        },
        Sw { rs1, rs2, imm } => {
            let addr = cpu.read_reg(rs1).wrapping_add(imm as u32);
            cpu.memory.write_u32(addr, cpu.read_reg(rs2))?;
        },
        Sh { rs1, rs2, imm } => {
            let addr = cpu.read_reg(rs1).wrapping_add(imm as u32);
            let val = cpu.read_reg(rs2) as u16;
            cpu.memory.write_u16(addr, val)?;
        },
        Sb { rs1, rs2, imm } => {
            let addr = cpu.read_reg(rs1).wrapping_add(imm as u32);
            let val = cpu.read_reg(rs2) as u8;
            cpu.memory.write_u8(addr, val)?;
        },
        Fence => {},
        Ecall => {
            let result = syscall::handle(cpu)?;
            if result != StepResult::Continue {
                return Ok(result);
            }
        },
        Ebreak => return Err(ErrorType::Breakpoint(pc)),
    }

    cpu.pc = next_pc;
    Ok(StepResult::Continue)
}

fn jump_target(target: u32) -> Result<u32, ErrorType> {
    if target % 4 != 0 {
        return Err(ErrorType::InstructionAddressMisaligned(target));
    }
    Ok(target)
}