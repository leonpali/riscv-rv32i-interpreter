use riscv_rv32i_interpreter::error::ErrorType;
use riscv_rv32i_interpreter::memory::Memory;

const SIZE: usize = 1024;
const END: u32 = SIZE as u32;

fn memory() -> Memory {
    Memory::new(SIZE)
}

// ---------- new ----------

#[test]
fn new_memory_is_zeroed() {
    let mem = memory();
    assert_eq!(mem.read_u32(0), Ok(0));
    assert_eq!(mem.read_u32(END - 4), Ok(0));
}

#[test]
fn new_memory_has_requested_size() {
    let mem = memory();
    assert_eq!(mem.read_u8(END - 1), Ok(0));
    assert_eq!(mem.read_u8(END), Err(ErrorType::MemoryOutOfBounds(END)));
}

#[test]
fn zero_sized_memory() {
    let mem = Memory::new(0);
    assert_eq!(mem.read_u8(0), Err(ErrorType::MemoryOutOfBounds(0)));
}

// ---------- u8 ----------

#[test]
fn u8_round_trip() {
    let mut mem = memory();
    mem.write_u8(0x10, 0xab).unwrap();
    mem.write_u8(0x11, 0xff).unwrap();
    assert_eq!(mem.read_u8(0x10), Ok(0xab));
    assert_eq!(mem.read_u8(0x11), Ok(0xff));
    assert_eq!(mem.read_u8(0x12), Ok(0));
}

#[test]
fn u8_last_byte() {
    let mut mem = memory();
    assert_eq!(mem.write_u8(END - 1, 7), Ok(()));
    assert_eq!(mem.read_u8(END - 1), Ok(7));
}

#[test]
fn u8_out_of_bounds() {
    let mut mem = memory();
    assert_eq!(mem.read_u8(END), Err(ErrorType::MemoryOutOfBounds(END)));
    assert_eq!(mem.write_u8(END, 1), Err(ErrorType::MemoryOutOfBounds(END)));
    assert_eq!(mem.read_u8(u32::MAX), Err(ErrorType::MemoryOutOfBounds(u32::MAX)));
}

// ---------- u16 ----------

#[test]
fn u16_is_little_endian() {
    let mut mem = memory();
    mem.write_u16(0x10, 0xbeef).unwrap();
    assert_eq!(mem.read_u8(0x10), Ok(0xef));
    assert_eq!(mem.read_u8(0x11), Ok(0xbe));
    assert_eq!(mem.read_u16(0x10), Ok(0xbeef));
}

#[test]
fn u16_reads_bytes_little_endian() {
    let mut mem = memory();
    mem.write_u8(0x10, 0x34).unwrap();
    mem.write_u8(0x11, 0x12).unwrap();
    assert_eq!(mem.read_u16(0x10), Ok(0x1234));
}

#[test]
fn u16_misaligned_access_is_allowed() {
    let mut mem = memory();
    mem.write_u16(0x11, 0xabcd).unwrap();
    assert_eq!(mem.read_u16(0x11), Ok(0xabcd));
    assert_eq!(mem.read_u8(0x11), Ok(0xcd));
    assert_eq!(mem.read_u8(0x12), Ok(0xab));
}

#[test]
fn u16_last_halfword() {
    let mut mem = memory();
    assert_eq!(mem.write_u16(END - 2, 0x1234), Ok(()));
    assert_eq!(mem.read_u16(END - 2), Ok(0x1234));
}

#[test]
fn u16_partially_out_of_bounds() {
    let mut mem = memory();
    assert_eq!(mem.read_u16(END - 1), Err(ErrorType::MemoryOutOfBounds(END - 1)));
    assert_eq!(
        mem.write_u16(END - 1, 0xffff),
        Err(ErrorType::MemoryOutOfBounds(END - 1))
    );
    // the byte that would have fit is not written
    assert_eq!(mem.read_u8(END - 1), Ok(0));
}

#[test]
fn u16_out_of_bounds() {
    let mut mem = memory();
    assert_eq!(mem.read_u16(END), Err(ErrorType::MemoryOutOfBounds(END)));
    assert_eq!(mem.write_u16(END, 1), Err(ErrorType::MemoryOutOfBounds(END)));
    assert_eq!(mem.read_u16(u32::MAX), Err(ErrorType::MemoryOutOfBounds(u32::MAX)));
}

// ---------- u32 ----------

#[test]
fn u32_is_little_endian() {
    let mut mem = memory();
    mem.write_u32(0x20, 0x12345678).unwrap();
    assert_eq!(mem.read_bytes(0x20, 4), Ok(&[0x78, 0x56, 0x34, 0x12][..]));
    assert_eq!(mem.read_u32(0x20), Ok(0x12345678));
}

#[test]
fn u32_reads_bytes_little_endian() {
    let mut mem = memory();
    mem.write_bytes(0x20, &[0x78, 0x56, 0x34, 0x12]).unwrap();
    assert_eq!(mem.read_u32(0x20), Ok(0x12345678));
}

#[test]
fn u32_misaligned_access_is_allowed() {
    let mut mem = memory();
    mem.write_u32(0x21, 0xdeadbeef).unwrap();
    assert_eq!(mem.read_u32(0x21), Ok(0xdeadbeef));
    assert_eq!(mem.read_u8(0x20), Ok(0));
    assert_eq!(mem.read_u8(0x25), Ok(0));
}

#[test]
fn u32_last_word() {
    let mut mem = memory();
    assert_eq!(mem.write_u32(END - 4, 0xcafebabe), Ok(()));
    assert_eq!(mem.read_u32(END - 4), Ok(0xcafebabe));
}

#[test]
fn u32_partially_out_of_bounds() {
    let mut mem = memory();
    assert_eq!(mem.read_u32(END - 3), Err(ErrorType::MemoryOutOfBounds(END - 3)));
    assert_eq!(
        mem.write_u32(END - 3, 0xffffffff),
        Err(ErrorType::MemoryOutOfBounds(END - 3))
    );
    // the 3 bytes that would have fit are not written
    assert_eq!(mem.read_bytes(END - 3, 3), Ok(&[0, 0, 0][..]));
}

#[test]
fn u32_out_of_bounds() {
    let mut mem = memory();
    assert_eq!(mem.read_u32(END), Err(ErrorType::MemoryOutOfBounds(END)));
    assert_eq!(mem.write_u32(END, 1), Err(ErrorType::MemoryOutOfBounds(END)));
    assert_eq!(mem.read_u32(u32::MAX), Err(ErrorType::MemoryOutOfBounds(u32::MAX)));
}

// ---------- mixed sizes ----------

#[test]
fn different_sizes_see_the_same_bytes() {
    let mut mem = memory();
    mem.write_u32(0x40, 0x11223344).unwrap();
    assert_eq!(mem.read_u16(0x40), Ok(0x3344));
    assert_eq!(mem.read_u16(0x42), Ok(0x1122));
    assert_eq!(mem.read_u8(0x43), Ok(0x11));

    mem.write_u16(0x42, 0xaaaa).unwrap();
    mem.write_u8(0x40, 0xbb).unwrap();
    assert_eq!(mem.read_u32(0x40), Ok(0xaaaa33bb));
}

// ---------- read_bytes ----------

#[test]
fn read_bytes_returns_bytes_in_memory_order() {
    let mut mem = memory();
    mem.write_u32(0x10, 0x44332211).unwrap();
    assert_eq!(mem.read_bytes(0x10, 4), Ok(&[0x11, 0x22, 0x33, 0x44][..]));
}

#[test]
fn read_bytes_partial_range() {
    let mut mem = memory();
    mem.write_u32(0x10, 0x44332211).unwrap();
    assert_eq!(mem.read_bytes(0x11, 2), Ok(&[0x22, 0x33][..]));
}

#[test]
fn read_bytes_zero_length() {
    let mem = memory();
    assert_eq!(mem.read_bytes(0x10, 0), Ok(&[][..]));
}

#[test]
fn read_bytes_all_of_memory() {
    let mem = memory();
    let bytes = mem.read_bytes(0, SIZE as u32).unwrap();
    assert_eq!(bytes.len(), SIZE);
    assert!(bytes.iter().all(|&b| b == 0));
}

#[test]
fn read_bytes_past_end() {
    let mem = memory();
    assert_eq!(mem.read_bytes(1020, 8), Err(ErrorType::MemoryOutOfBounds(1020)));
}

#[test]
fn read_bytes_start_out_of_bounds() {
    let mem = memory();
    assert_eq!(mem.read_bytes(2000, 1), Err(ErrorType::MemoryOutOfBounds(2000)));
}

#[test]
fn read_bytes_huge_length_does_not_panic() {
    let mem = memory();
    assert_eq!(mem.read_bytes(0, u32::MAX), Err(ErrorType::MemoryOutOfBounds(0)));
    assert_eq!(
        mem.read_bytes(u32::MAX, u32::MAX),
        Err(ErrorType::MemoryOutOfBounds(u32::MAX))
    );
}

// ---------- write_bytes ----------

#[test]
fn write_bytes_stores_each_byte() {
    let mut mem = memory();
    mem.write_bytes(0x10, &[1, 2, 3]).unwrap();
    assert_eq!(mem.read_u8(0x10), Ok(1));
    assert_eq!(mem.read_u8(0x11), Ok(2));
    assert_eq!(mem.read_u8(0x12), Ok(3));
    // neighbours untouched
    assert_eq!(mem.read_u8(0x0f), Ok(0));
    assert_eq!(mem.read_u8(0x13), Ok(0));
}

#[test]
fn write_bytes_keeps_byte_order() {
    // bytes are copied as-is, not reversed: these are the
    // little-endian bytes of addi x1, x0, 5
    let mut mem = memory();
    mem.write_bytes(0, &[0x93, 0x00, 0x50, 0x00]).unwrap();
    assert_eq!(mem.read_u32(0), Ok(0x00500093));
}

#[test]
fn write_bytes_then_read_bytes_round_trips() {
    let mut mem = memory();
    mem.write_bytes(0x100, b"hello").unwrap();
    assert_eq!(mem.read_bytes(0x100, 5), Ok(&b"hello"[..]));
}

#[test]
fn write_bytes_overwrites() {
    let mut mem = memory();
    mem.write_u32(0x10, 0xffffffff).unwrap();
    mem.write_bytes(0x11, &[0, 0]).unwrap();
    assert_eq!(mem.read_u32(0x10), Ok(0xff0000ff));
}

#[test]
fn write_bytes_empty_slice() {
    let mut mem = memory();
    mem.write_u8(0x10, 7).unwrap();
    assert_eq!(mem.write_bytes(0x10, &[]), Ok(()));
    assert_eq!(mem.read_u8(0x10), Ok(7));
}

#[test]
fn write_bytes_all_of_memory() {
    let mut mem = memory();
    assert_eq!(mem.write_bytes(0, &[0xab; SIZE]), Ok(()));
    assert_eq!(mem.read_u8(0), Ok(0xab));
    assert_eq!(mem.read_u8(SIZE as u32 - 1), Ok(0xab));
}

#[test]
fn write_bytes_past_end_writes_nothing() {
    let mut mem = memory();
    assert_eq!(
        mem.write_bytes(1020, &[1; 8]),
        Err(ErrorType::MemoryOutOfBounds(1020))
    );
    // no partial write of the 4 bytes that would have fit
    assert_eq!(mem.read_bytes(1020, 4), Ok(&[0, 0, 0, 0][..]));
}

#[test]
fn write_bytes_start_out_of_bounds() {
    let mut mem = memory();
    assert_eq!(
        mem.write_bytes(2000, &[1]),
        Err(ErrorType::MemoryOutOfBounds(2000))
    );
}
