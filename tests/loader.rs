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

// ---------- ELF ----------

use riscv_rv32i_interpreter::loader::{is_elf, load_elf};

const PT_LOAD: u32 = 1;
const PT_NOTE: u32 = 4;
const PHOFF: u32 = 52;

struct Seg<'a> {
    kind: u32,
    addr: u32,
    data: &'a [u8],
    memsz: u32,
}

fn seg(addr: u32, data: &[u8]) -> Seg<'_> {
    Seg { kind: PT_LOAD, addr, data, memsz: data.len() as u32 }
}

/// Builds a minimal 32-bit little-endian RISC-V ELF executable:
/// ELF header, then the program headers, then the segment data.
fn elf(entry: u32, segs: &[Seg]) -> Vec<u8> {
    let mut f = vec![0u8; 52];
    f[..4].copy_from_slice(&[0x7f, b'E', b'L', b'F']);
    f[4] = 1; // ELFCLASS32
    f[5] = 1; // ELFDATA2LSB
    f[6] = 1; // EV_CURRENT
    set_u16(&mut f, 0x10, 2); // ET_EXEC
    set_u16(&mut f, 0x12, 0xf3); // EM_RISCV
    set_u32(&mut f, 0x14, 1); // e_version
    set_u32(&mut f, 0x18, entry);
    set_u32(&mut f, 0x1c, PHOFF);
    set_u16(&mut f, 0x28, 52); // e_ehsize
    set_u16(&mut f, 0x2a, 32); // e_phentsize
    set_u16(&mut f, 0x2c, segs.len() as u16);

    let mut offset = PHOFF + 32 * segs.len() as u32;
    for s in segs {
        for field in [s.kind, offset, s.addr, s.addr, s.data.len() as u32, s.memsz, 7, 4] {
            f.extend_from_slice(&field.to_le_bytes());
        }
        offset += s.data.len() as u32;
    }
    for s in segs {
        f.extend_from_slice(s.data);
    }
    f
}

fn set_u16(f: &mut [u8], at: usize, v: u16) {
    f[at..at + 2].copy_from_slice(&v.to_le_bytes());
}

fn set_u32(f: &mut [u8], at: usize, v: u32) {
    f[at..at + 4].copy_from_slice(&v.to_le_bytes());
}

fn invalid(msg: &'static str) -> Result<(), ErrorType> {
    Err(ErrorType::InvalidElf(msg))
}

#[test]
fn elf_loads_segment_and_sets_entry() {
    let mut cpu = Cpu::new(1024);
    load_elf(&mut cpu, &elf(0x100, &[seg(0x100, &[1, 2, 3, 4])])).unwrap();
    assert_eq!(cpu.memory.read_bytes(0x100, 4), Ok(&[1, 2, 3, 4][..]));
    assert_eq!(cpu.memory.read_u8(0x104), Ok(0));
    assert_eq!(cpu.pc, 0x100);
}

#[test]
fn elf_entry_need_not_be_segment_start() {
    let mut cpu = Cpu::new(1024);
    load_elf(&mut cpu, &elf(0x108, &[seg(0x100, &[0; 16])])).unwrap();
    assert_eq!(cpu.pc, 0x108);
}

#[test]
fn elf_loads_multiple_segments() {
    let mut cpu = Cpu::new(1024);
    load_elf(&mut cpu, &elf(0, &[seg(0x100, &[1, 2]), seg(0x200, &[3, 4])])).unwrap();
    assert_eq!(cpu.memory.read_bytes(0x100, 2), Ok(&[1, 2][..]));
    assert_eq!(cpu.memory.read_bytes(0x200, 2), Ok(&[3, 4][..]));
}

#[test]
fn elf_zero_fills_rest_of_segment() {
    // memsz > filesz: the extra bytes (.bss) must be zeroed
    let mut cpu = Cpu::new(1024);
    cpu.memory.write_bytes(0x100, &[0xff; 16]).unwrap();
    let s = Seg { kind: PT_LOAD, addr: 0x100, data: &[1, 2, 3, 4], memsz: 12 };
    load_elf(&mut cpu, &elf(0x100, &[s])).unwrap();
    assert_eq!(cpu.memory.read_bytes(0x100, 4), Ok(&[1, 2, 3, 4][..]));
    assert_eq!(cpu.memory.read_bytes(0x104, 8), Ok(&[0; 8][..]));
    // past memsz is untouched
    assert_eq!(cpu.memory.read_bytes(0x10c, 4), Ok(&[0xff; 4][..]));
}

#[test]
fn elf_skips_non_load_segments() {
    let mut cpu = Cpu::new(1024);
    let note = Seg { kind: PT_NOTE, addr: 0x300, data: &[9, 9, 9, 9], memsz: 4 };
    load_elf(&mut cpu, &elf(0x100, &[seg(0x100, &[1]), note])).unwrap();
    assert_eq!(cpu.memory.read_bytes(0x300, 4), Ok(&[0; 4][..]));
}

#[test]
fn elf_uses_physical_address() {
    let mut cpu = Cpu::new(1024);
    let mut file = elf(0x100, &[seg(0x100, &[1, 2, 3, 4])]);
    set_u32(&mut file, PHOFF as usize + 8, 0x200); // p_vaddr differs from p_paddr
    load_elf(&mut cpu, &file).unwrap();
    assert_eq!(cpu.memory.read_bytes(0x100, 4), Ok(&[1, 2, 3, 4][..]));
    assert_eq!(cpu.memory.read_bytes(0x200, 4), Ok(&[0; 4][..]));
}

#[test]
fn is_elf_detects_magic() {
    assert!(is_elf(&elf(0, &[])));
    assert!(!is_elf(&to_bytes(&[0x00500093])));
    assert!(!is_elf(&[]));
}

#[test]
fn elf_rejects_non_elf_files() {
    let mut cpu = Cpu::new(1024);
    assert_eq!(load_elf(&mut cpu, &to_bytes(&[0x00500093])), invalid("not an ELF file"));
    // just the magic is too short to be a header
    assert_eq!(load_elf(&mut cpu, &[0x7f, b'E', b'L', b'F']), invalid("not an ELF file"));
}

#[test]
fn elf_rejects_wrong_format() {
    let mut cpu = Cpu::new(1024);
    let good = elf(0, &[]);

    let mut f = good.clone();
    f[4] = 2; // ELFCLASS64
    assert_eq!(load_elf(&mut cpu, &f), invalid("not a 32-bit ELF file"));

    let mut f = good.clone();
    f[5] = 2; // big-endian
    assert_eq!(load_elf(&mut cpu, &f), invalid("not a little-endian ELF file"));

    let mut f = good.clone();
    set_u16(&mut f, 0x10, 1); // ET_REL, e.g. an unlinked .o file
    assert_eq!(load_elf(&mut cpu, &f), invalid("not an executable"));

    let mut f = good.clone();
    set_u16(&mut f, 0x12, 0x3e); // x86-64
    assert_eq!(load_elf(&mut cpu, &f), invalid("not a RISC-V executable"));
}

#[test]
fn elf_rejects_truncated_program_headers() {
    let mut cpu = Cpu::new(1024);
    let mut f = elf(0x100, &[seg(0x100, &[1, 2, 3, 4])]);
    f.truncate(60);
    assert_eq!(load_elf(&mut cpu, &f), invalid("truncated header"));
}

#[test]
fn elf_rejects_segment_data_past_end_of_file() {
    let mut cpu = Cpu::new(1024);
    let mut f = elf(0x100, &[seg(0x100, &[1, 2, 3, 4])]);
    set_u32(&mut f, PHOFF as usize + 4, 0xffff_fff0); // p_offset
    assert_eq!(load_elf(&mut cpu, &f), invalid("segment data past end of file"));
}

#[test]
fn elf_rejects_filesz_larger_than_memsz() {
    let mut cpu = Cpu::new(1024);
    let s = Seg { kind: PT_LOAD, addr: 0x100, data: &[1, 2, 3, 4], memsz: 2 };
    assert_eq!(
        load_elf(&mut cpu, &elf(0x100, &[s])),
        invalid("segment larger in file than in memory")
    );
}

#[test]
fn elf_segment_outside_memory() {
    let mut cpu = Cpu::new(1024);
    assert_eq!(
        load_elf(&mut cpu, &elf(0, &[seg(1020, &[0; 8])])),
        Err(ErrorType::MemoryOutOfBounds(1020))
    );
}

#[test]
fn elf_misaligned_entry() {
    let mut cpu = Cpu::new(1024);
    assert_eq!(
        load_elf(&mut cpu, &elf(0x102, &[seg(0x100, &[0; 8])])),
        Err(ErrorType::InstructionAddressMisaligned(0x102))
    );
}

#[test]
fn failed_elf_load_writes_nothing() {
    // the first segment fits, the second doesn't: memory and pc stay untouched
    let mut cpu = Cpu::new(1024);
    cpu.pc = 0x40;
    let file = elf(0x100, &[seg(0x100, &[1, 2, 3, 4]), seg(1020, &[0; 8])]);
    assert!(load_elf(&mut cpu, &file).is_err());
    assert_eq!(cpu.memory.read_bytes(0x100, 4), Ok(&[0; 4][..]));
    assert_eq!(cpu.pc, 0x40);
}

#[test]
fn loaded_elf_runs() {
    let code = to_bytes(&[
        0x02a00513, // addi a0, x0, 42
        0x05d00893, // addi a7, x0, 93  (exit)
        0x00000073, // ecall
    ]);
    let mut cpu = Cpu::new(1024);
    load_elf(&mut cpu, &elf(0x100, &[seg(0x100, &code)])).unwrap();
    for _ in 0..2 {
        assert_eq!(cpu.step(), Ok(StepResult::Continue));
    }
    assert_eq!(cpu.step(), Ok(StepResult::Exit(42)));
}

// ---------- memory at a base address ----------

const BASE: u32 = 0x8000_0000;

#[test]
fn load_at_memory_base() {
    let mut cpu = Cpu::with_memory_base(BASE, 1024);
    load(&mut cpu, &[1, 2, 3, 4], BASE).unwrap();
    assert_eq!(cpu.memory.read_bytes(BASE, 4), Ok(&[1, 2, 3, 4][..]));
    assert_eq!(cpu.pc, BASE);
}

#[test]
fn load_below_memory_base() {
    let mut cpu = Cpu::with_memory_base(BASE, 1024);
    assert_eq!(load(&mut cpu, &[1, 2, 3, 4], 0), Err(ErrorType::MemoryOutOfBounds(0)));
}

#[test]
fn elf_at_memory_base() {
    let mut cpu = Cpu::with_memory_base(BASE, 1024);
    load_elf(&mut cpu, &elf(BASE + 8, &[seg(BASE, &[1, 2, 3, 4])])).unwrap();
    assert_eq!(cpu.memory.read_bytes(BASE, 4), Ok(&[1, 2, 3, 4][..]));
    assert_eq!(cpu.pc, BASE + 8);
}

#[test]
fn elf_segment_below_memory_base() {
    // e.g. a segment that also maps the ELF headers, one page below RAM
    let mut cpu = Cpu::with_memory_base(BASE, 1024);
    assert_eq!(
        load_elf(&mut cpu, &elf(BASE, &[seg(BASE - 0x1000, &[0; 8])])),
        Err(ErrorType::MemoryOutOfBounds(BASE - 0x1000))
    );
}
