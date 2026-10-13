use crate::error::ErrorType;
use crate::instruction::Instruction;
use crate::instruction::Instruction::*;

pub fn decode(raw: u32) -> Result<Instruction, ErrorType> {
    let illegal = Err(ErrorType::IllegalInstruction(raw));
    let (rd, rs1, rs2) = (rd(raw), rs1(raw), rs2(raw));

    let inst = match opcode(raw) {
        0b0110111 => Lui { rd, imm: imm_u(raw) },
        0b0010111 => Auipc { rd, imm: imm_u(raw) },
        0b1101111 => Jal { rd, imm: imm_j(raw) },

        0b1100111 => match funct3(raw) {
            0b000 => Jalr { rd, rs1, imm: imm_i(raw) },
            _ => return illegal,
        }

        0b1100011 => {
            let imm = imm_b(raw);
            match funct3(raw) {
                0b000 => Beq { rs1, rs2, imm },
                0b001 => Bne { rs1, rs2, imm },
                0b100 => Blt { rs1, rs2, imm },
                0b101 => Bge { rs1, rs2, imm },
                0b110 => Bltu { rs1, rs2, imm },
                0b111 => Bgeu { rs1, rs2, imm },
                _ => return illegal,
            }
        }

        0b0000011 => {
            let imm = imm_i(raw);
            match funct3(raw) {
                0b000 => Lb { rd, rs1, imm },
                0b001 => Lh { rd, rs1, imm },
                0b010 => Lw { rd, rs1, imm },
                0b100 => Lbu { rd, rs1, imm },
                0b101 => Lhu { rd, rs1, imm },
                _ => return illegal,
            }
        }

        0b0100011 => {
            let imm = imm_s(raw);
            match funct3(raw) {
                0b000 => Sb { rs1, rs2, imm },
                0b001 => Sh { rs1, rs2, imm },
                0b010 => Sw { rs1, rs2, imm },
                _ => return illegal,
            }
        }

        0b0010011 => {
            let imm = imm_i(raw);
            let shamt = rs2 as i32;
            match funct3(raw) {
                0b000 => Addi { rd, rs1, imm },
                0b010 => Slti { rd, rs1, imm },
                0b011 => Sltiu { rd, rs1, imm },
                0b100 => Xori { rd, rs1, imm },
                0b110 => Ori { rd, rs1, imm },
                0b111 => Andi { rd, rs1, imm },
                0b001 if funct7(raw) == 0b0000000 => Slli { rd, rs1, shamt },
                0b101 if funct7(raw) == 0b0000000 => Srli { rd, rs1, shamt },
                0b101 if funct7(raw) == 0b0100000 => Srai { rd, rs1, shamt },
                _ => return illegal,
            }
        }

        0b0110011 => {
            let funct7 = funct7(raw);
            match funct3(raw) {
                0b000 if funct7 == 0b0000000 => Add { rd, rs1, rs2 },
                0b000 if funct7 == 0b0100000 => Sub { rd, rs1, rs2 },
                0b001 if funct7 == 0b0000000 => Sll { rd, rs1, rs2 },
                0b010 if funct7 == 0b0000000 => Slt { rd, rs1, rs2 },
                0b011 if funct7 == 0b0000000 => Sltu { rd, rs1, rs2 },
                0b100 if funct7 == 0b0000000 => Xor { rd, rs1, rs2 },
                0b101 if funct7 == 0b0000000 => Srl { rd, rs1, rs2 },
                0b101 if funct7 == 0b0100000 => Sra { rd, rs1, rs2 },
                0b110 if funct7 == 0b0000000 => Or { rd, rs1, rs2 },
                0b111 if funct7 == 0b0000000 => And { rd, rs1, rs2 },
                _ => return illegal,
            }
        }

        0b0001111 => Fence,

        0b1110011 => {
            match imm_i(raw) {
                0 => Ecall,
                1 => Ebreak,
                _ => return illegal,
            }
        }

        _ => return illegal,
    };
    Ok(inst)
}

fn opcode(raw: u32) -> u32 { raw & 0x7f }

fn rd(raw: u32) -> u8 { ((raw >> 7) & 0x1f) as u8 }

fn funct3(raw: u32) -> u32 { (raw >> 12) & 0x7 }

fn rs1(raw: u32) -> u8 { ((raw >> 15) & 0x1f) as u8 }

fn rs2(raw: u32) -> u8 { ((raw >> 20) & 0x1f) as u8 }

fn funct7(raw: u32) -> u32 { raw >> 25 }

fn imm_i(raw: u32) -> i32 { (raw as i32) >> 20 }

fn imm_s(raw: u32) -> i32 {
    let high = (raw as i32) >> 25;
    let low = ((raw >> 7) & 0x1f) as i32;
    (high << 5) | low
}

fn imm_b(raw: u32) -> i32 {
    let high = ((raw >> 25) & 0x3f) as i32;
    let low = ((raw >> 8) & 0xF) as i32;
    let sign = ((raw as i32) >> 31) << 12;
    let bit11 = ((raw >> 7) & 0x1) as i32;
    sign | (bit11 << 11) | (high << 5) | (low << 1)
}

fn imm_u(raw: u32) -> i32 { (raw & 0xfffff000) as i32 }

fn imm_j(raw: u32) -> i32 {
    let sign = ((raw as i32) >> 31) << 20;
    let bit19to12 = ((raw >> 12) & 0xFF) as i32;
    let bit11 = ((raw >> 20) & 0x1) as i32;
    let bit10to1 = ((raw >> 21) & 0x3FF) as i32;
    sign | (bit10to1 << 1) | (bit11 << 11) | (bit19to12 << 12)
}