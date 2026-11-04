use crate::cpu::Cpu;
use crate::error::ErrorType;

const INSTRUCTION_ALIGNMENT: u32 = 4;

pub const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

const ELF_HEADER_SIZE: usize = 52;
const PROGRAM_HEADER_SIZE: usize = 32;

const EI_CLASS: usize = 0x04;
const EI_DATA: usize = 0x05;
const E_TYPE: usize = 0x10;
const E_MACHINE: usize = 0x12;
const E_ENTRY: usize = 0x18;
const E_PHOFF: usize = 0x1c;
const E_PHENTSIZE: usize = 0x2a;
const E_PHNUM: usize = 0x2c;

const P_TYPE: usize = 0x00;
const P_OFFSET: usize = 0x04;
const P_PADDR: usize = 0x0c;
const P_FILESZ: usize = 0x10;
const P_MEMSZ: usize = 0x14;

const ELFCLASS32: u8 = 1;
const ELFDATA2LSB: u8 = 1;
const ET_EXEC: u16 = 2;
const EM_RISCV: u16 = 0xf3;
const PT_LOAD: u32 = 1;

pub fn load(cpu: &mut Cpu, program: &[u8], base: u32) -> Result<(), ErrorType> {
    if base % INSTRUCTION_ALIGNMENT != 0 {
        return Err(ErrorType::InstructionAddressMisaligned(base));
    }
    cpu.memory.write_bytes(base, program)?;
    cpu.pc = base;
    Ok(())
}

pub fn is_elf(file: &[u8]) -> bool {
    file.starts_with(&ELF_MAGIC)
}

struct Segment<'a> {
    addr: u32,
    data: &'a [u8],
    zeros: u32,
}

pub fn load_elf(cpu: &mut Cpu, file: &[u8]) -> Result<(), ErrorType> {
    if file.len() < ELF_HEADER_SIZE || !is_elf(file) {
        return Err(ErrorType::InvalidElf("not an ELF file"));
    }
    if file[EI_CLASS] != ELFCLASS32 {
        return Err(ErrorType::InvalidElf("not a 32-bit ELF file"));
    }
    if file[EI_DATA] != ELFDATA2LSB {
        return Err(ErrorType::InvalidElf("not a little-endian ELF file"));
    }
    if read_u16(file, E_TYPE)? != ET_EXEC {
        return Err(ErrorType::InvalidElf("not an executable"));
    }
    if read_u16(file, E_MACHINE)? != EM_RISCV {
        return Err(ErrorType::InvalidElf("not a RISC-V executable"));
    }

    let entry = read_u32(file, E_ENTRY)?;
    let phoff = read_u32(file, E_PHOFF)? as usize;
    let phentsize = read_u16(file, E_PHENTSIZE)? as usize;
    let phnum = read_u16(file, E_PHNUM)? as usize;
    if phnum > 0 && phentsize != PROGRAM_HEADER_SIZE {
        return Err(ErrorType::InvalidElf("unexpected program header size"));
    }
    if entry % INSTRUCTION_ALIGNMENT != 0 {
        return Err(ErrorType::InstructionAddressMisaligned(entry));
    }

    let mut segments = Vec::new();
    for i in 0..phnum {
        let ph = phoff + i * PROGRAM_HEADER_SIZE;
        if read_u32(file, ph + P_TYPE)? != PT_LOAD {
            continue;
        }
        let offset = read_u32(file, ph + P_OFFSET)? as usize;
        let paddr = read_u32(file, ph + P_PADDR)?;
        let filesz = read_u32(file, ph + P_FILESZ)?;
        let memsz = read_u32(file, ph + P_MEMSZ)?;
        if filesz > memsz {
            return Err(ErrorType::InvalidElf("segment larger in file than in memory"));
        }
        let data = offset
            .checked_add(filesz as usize)
            .and_then(|end| file.get(offset..end))
            .ok_or(ErrorType::InvalidElf("segment data past end of file"))?;
        cpu.memory.read_bytes(paddr, memsz)?;
        segments.push(Segment { addr: paddr, data, zeros: memsz - filesz });
    }

    for segment in segments {
        cpu.memory.write_bytes(segment.addr, segment.data)?;
        let bss = segment.addr.wrapping_add(segment.data.len() as u32);
        cpu.memory.write_bytes(bss, &vec![0; segment.zeros as usize])?;
    }
    cpu.pc = entry;
    Ok(())
}

fn read_u16(file: &[u8], offset: usize) -> Result<u16, ErrorType> {
    file.get(offset..offset + size_of::<u16>())
        .map(|b| u16::from_le_bytes(b.try_into().unwrap()))
        .ok_or(ErrorType::InvalidElf("truncated header"))
}

fn read_u32(file: &[u8], offset: usize) -> Result<u32, ErrorType> {
    file.get(offset..offset + size_of::<u32>())
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        .ok_or(ErrorType::InvalidElf("truncated header"))
}
