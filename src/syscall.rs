use crate::cpu::{Cpu, StepResult};
use crate::error::ErrorType;

pub fn handle(cpu: &mut Cpu) -> Result<StepResult, ErrorType> {
    let syscall = cpu.read_reg(17);
    match syscall {
        93 => return Ok(StepResult::Exit(cpu.read_reg(10))),
        64 => {
            let out = cpu.read_reg(10);
            match out {
                1 => {
                    let len = cpu.read_reg(12);
                    let buf = cpu.read_reg(11);
                    let bytes = cpu.memory.read_bytes(buf, len)?;
                    cpu.output.extend_from_slice(bytes);
                    cpu.write_reg(10, len);
                },
                _ => cpu.write_reg(10, -9i32 as u32),
            }
        },
        _ => return Err(ErrorType::UnsupportedSyscall(syscall)),
    }
    Ok(StepResult::Continue)
}
