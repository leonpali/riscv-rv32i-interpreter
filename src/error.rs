#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    IllegalInstruction(u32),
    MemoryOutOfBounds(u32),
}
