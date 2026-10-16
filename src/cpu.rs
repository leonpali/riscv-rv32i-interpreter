use crate::memory::Memory;
use crate::error::ErrorType;
use crate::decode::decode;
use crate::execute::execute;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepResult {
    Continue,
    Exit(u32),
}

pub struct Cpu {
    pub registers: [u32; 32],
    pub pc: u32,
    pub memory: Memory,
    pub output: Vec<u8>,
}

impl Cpu {
    pub fn new(memory_size: usize) -> Self {
        Self {
            registers: [0; 32],
            pc: 0,
            memory: Memory::new(memory_size),
            output: Vec::new(),
        }
    }

    pub fn step(&mut self) -> Result<StepResult, ErrorType> {
        let raw = self.memory.read_u32(self.pc)?;
        let inst = decode(raw)?;
        execute(self, inst)
    }

    pub fn read_reg(&self, reg: u8) -> u32 {
        self.registers[reg as usize]
    }

    pub fn write_reg(&mut self, reg: u8, val: u32) {
        if reg != 0 {
            self.registers[reg as usize] = val;
        }
    }
}