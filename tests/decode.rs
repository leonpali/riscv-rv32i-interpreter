use riscv_rv32i_interpreter::decode::decode;
use riscv_rv32i_interpreter::error::ErrorType;
use riscv_rv32i_interpreter::instruction::Instruction::*;

// Expected encodings come from riscv64-unknown-elf-as -march=rv32i.
// Immediates are the final sign-extended values: U-type imm is already
// shifted left by 12, B/J-type imm is the byte offset from pc.

#[test]
fn u_type() {
    // lui x1, 0x12345
    assert_eq!(decode(0x123450b7), Ok(Lui { rd: 1, imm: 0x12345000 }));
    // lui x31, 0xfffff
    assert_eq!(decode(0xffffffb7), Ok(Lui { rd: 31, imm: -4096 }));
    // auipc x5, 0x80000
    assert_eq!(decode(0x80000297), Ok(Auipc { rd: 5, imm: i32::MIN }));
}

#[test]
fn jal() {
    // jal x1, +2048
    assert_eq!(decode(0x001000ef), Ok(Jal { rd: 1, imm: 2048 }));
    // jal x0, -4
    assert_eq!(decode(0xffdff06f), Ok(Jal { rd: 0, imm: -4 }));
    // jal x1, +0xffffe (largest offset)
    assert_eq!(decode(0x7ffff0ef), Ok(Jal { rd: 1, imm: 0xffffe }));
    // jal x1, -0x100000 (smallest offset)
    assert_eq!(decode(0x800000ef), Ok(Jal { rd: 1, imm: -0x100000 }));
}

#[test]
fn jalr() {
    // jalr x0, 0(x1)
    assert_eq!(decode(0x00008067), Ok(Jalr { rd: 0, rs1: 1, imm: 0 }));
    // jalr x5, -1(x6)
    assert_eq!(decode(0xfff302e7), Ok(Jalr { rd: 5, rs1: 6, imm: -1 }));
}

#[test]
fn branches() {
    // beq x1, x2, +16
    assert_eq!(decode(0x00208863), Ok(Beq { rs1: 1, rs2: 2, imm: 16 }));
    // bne x3, x4, -16
    assert_eq!(decode(0xfe4198e3), Ok(Bne { rs1: 3, rs2: 4, imm: -16 }));
    // blt x5, x6, +4094 (largest offset)
    assert_eq!(decode(0x7e62cfe3), Ok(Blt { rs1: 5, rs2: 6, imm: 4094 }));
    // bge x7, x8, -4096 (smallest offset)
    assert_eq!(decode(0x8083d063), Ok(Bge { rs1: 7, rs2: 8, imm: -4096 }));
    // bltu x9, x10, +8
    assert_eq!(decode(0x00a4e463), Ok(Bltu { rs1: 9, rs2: 10, imm: 8 }));
    // bgeu x11, x12, +2048 (only imm[11] set)
    assert_eq!(decode(0x00c5f0e3), Ok(Bgeu { rs1: 11, rs2: 12, imm: 2048 }));
}

#[test]
fn loads() {
    // lb x1, 0(x2)
    assert_eq!(decode(0x00010083), Ok(Lb { rd: 1, rs1: 2, imm: 0 }));
    // lh x3, -2(x4)
    assert_eq!(decode(0xffe21183), Ok(Lh { rd: 3, rs1: 4, imm: -2 }));
    // lw x5, 2047(x6)
    assert_eq!(decode(0x7ff32283), Ok(Lw { rd: 5, rs1: 6, imm: 2047 }));
    // lbu x7, -2048(x8)
    assert_eq!(decode(0x80044383), Ok(Lbu { rd: 7, rs1: 8, imm: -2048 }));
    // lhu x9, 4(x10)
    assert_eq!(decode(0x00455483), Ok(Lhu { rd: 9, rs1: 10, imm: 4 }));
}

#[test]
fn stores() {
    // sb x1, 0(x2)
    assert_eq!(decode(0x00110023), Ok(Sb { rs1: 2, rs2: 1, imm: 0 }));
    // sh x3, -2(x4)
    assert_eq!(decode(0xfe321f23), Ok(Sh { rs1: 4, rs2: 3, imm: -2 }));
    // sw x5, 2047(x6)
    assert_eq!(decode(0x7e532fa3), Ok(Sw { rs1: 6, rs2: 5, imm: 2047 }));
    // sw x31, -2048(x30)
    assert_eq!(decode(0x81ff2023), Ok(Sw { rs1: 30, rs2: 31, imm: -2048 }));
}

#[test]
fn register_immediate() {
    // addi x1, x0, 5
    assert_eq!(decode(0x00500093), Ok(Addi { rd: 1, rs1: 0, imm: 5 }));
    // addi x2, x3, -1
    assert_eq!(decode(0xfff18113), Ok(Addi { rd: 2, rs1: 3, imm: -1 }));
    // nop (addi x0, x0, 0)
    assert_eq!(decode(0x00000013), Ok(Addi { rd: 0, rs1: 0, imm: 0 }));
    // slti x4, x5, -2048
    assert_eq!(decode(0x8002a213), Ok(Slti { rd: 4, rs1: 5, imm: -2048 }));
    // sltiu x6, x7, 2047
    assert_eq!(decode(0x7ff3b313), Ok(Sltiu { rd: 6, rs1: 7, imm: 2047 }));
    // xori x8, x9, -1
    assert_eq!(decode(0xfff4c413), Ok(Xori { rd: 8, rs1: 9, imm: -1 }));
    // ori x10, x11, 0x555
    assert_eq!(decode(0x5555e513), Ok(Ori { rd: 10, rs1: 11, imm: 0x555 }));
    // andi x12, x13, 0xff
    assert_eq!(decode(0x0ff6f613), Ok(Andi { rd: 12, rs1: 13, imm: 0xff }));
}

#[test]
fn shift_immediate() {
    // slli x1, x2, 31
    assert_eq!(decode(0x01f11093), Ok(Slli { rd: 1, rs1: 2, shamt: 31 }));
    // srli x3, x4, 1
    assert_eq!(decode(0x00125193), Ok(Srli { rd: 3, rs1: 4, shamt: 1 }));
    // srai x5, x6, 31 (funct7 0100000 must not leak into shamt)
    assert_eq!(decode(0x41f35293), Ok(Srai { rd: 5, rs1: 6, shamt: 31 }));
}

#[test]
fn register_register() {
    // add x1, x2, x3
    assert_eq!(decode(0x003100b3), Ok(Add { rd: 1, rs1: 2, rs2: 3 }));
    // sub x4, x5, x6
    assert_eq!(decode(0x40628233), Ok(Sub { rd: 4, rs1: 5, rs2: 6 }));
    // sll x7, x8, x9
    assert_eq!(decode(0x009413b3), Ok(Sll { rd: 7, rs1: 8, rs2: 9 }));
    // slt x10, x11, x12
    assert_eq!(decode(0x00c5a533), Ok(Slt { rd: 10, rs1: 11, rs2: 12 }));
    // sltu x13, x14, x15
    assert_eq!(decode(0x00f736b3), Ok(Sltu { rd: 13, rs1: 14, rs2: 15 }));
    // xor x16, x17, x18
    assert_eq!(decode(0x0128c833), Ok(Xor { rd: 16, rs1: 17, rs2: 18 }));
    // srl x19, x20, x21
    assert_eq!(decode(0x015a59b3), Ok(Srl { rd: 19, rs1: 20, rs2: 21 }));
    // sra x22, x23, x24
    assert_eq!(decode(0x418bdb33), Ok(Sra { rd: 22, rs1: 23, rs2: 24 }));
    // or x25, x26, x27
    assert_eq!(decode(0x01bd6cb3), Ok(Or { rd: 25, rs1: 26, rs2: 27 }));
    // and x28, x29, x31
    assert_eq!(decode(0x01fefe33), Ok(And { rd: 28, rs1: 29, rs2: 31 }));
}

#[test]
fn system() {
    // fence iorw, iorw
    assert_eq!(decode(0x0ff0000f), Ok(Fence));
    assert_eq!(decode(0x00000073), Ok(Ecall));
    assert_eq!(decode(0x00100073), Ok(Ebreak));
}

#[test]
fn illegal_instructions() {
    let illegal = [
        0x00000000, // all zeros is defined as illegal
        0xffffffff, // all ones is defined as illegal
        0x0000007f, // unknown opcode
        0x00013083, // ld (RV64 only, LOAD funct3 = 011)
        0x00113023, // sd (RV64 only, STORE funct3 = 011)
        0x0020a063, // BRANCH funct3 = 010
        0x00009067, // JALR funct3 = 001
        0x023100b3, // mul (M extension, funct7 = 0000001)
        0x02011093, // slli shamt = 32 (RV64 only)
        0x20125193, // SRLI/SRAI with an invalid funct7
    ];
    for raw in illegal {
        assert_eq!(
            decode(raw),
            Err(ErrorType::IllegalInstruction(raw)),
            "{raw:#010x} should be illegal"
        );
    }
}
