use crate::cpu::Cpu;
use crate::error::ErrorType;

pub fn load(cpu: &mut Cpu, program: &[u8], base: u32) -> Result<(), ErrorType> {
    if base % 4 != 0 {
        return Err(ErrorType::InstructionAddressMisaligned(base));
    }
    cpu.memory.write_bytes(base, program)?;
    cpu.pc = base;
    Ok(())
}
