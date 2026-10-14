#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    IllegalInstruction(u32),
    MemoryOutOfBounds(u32),
    InstructionAddressMisaligned(u32),
    Breakpoint(u32),
    UnsupportedSyscall(u32),
}
