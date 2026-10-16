use crate::error::ErrorType;

pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Self { data: vec![0; size] }
    }

    fn slice(&self, addr: u32, len: usize) -> Result<&[u8], ErrorType> {
        let start = addr as usize;
        start
            .checked_add(len)
            .and_then(|end| self.data.get(start..end))
            .ok_or(ErrorType::MemoryOutOfBounds(addr))
    }

    fn slice_mut(&mut self, addr: u32, len: usize) -> Result<&mut [u8], ErrorType> {
        let start = addr as usize;
        start
            .checked_add(len)
            .and_then(|end| self.data.get_mut(start..end))
            .ok_or(ErrorType::MemoryOutOfBounds(addr))
    }

    pub fn read_u32(&self, addr: u32) -> Result<u32, ErrorType> {
        Ok(u32::from_le_bytes(self.slice(addr, 4)?.try_into().unwrap()))
    }

    pub fn write_u32(&mut self, addr: u32, val: u32) -> Result<(), ErrorType> {
        self.slice_mut(addr, 4)?.copy_from_slice(&val.to_le_bytes());
        Ok(())
    }

    pub fn read_u16(&self, addr: u32) -> Result<u16, ErrorType> {
        Ok(u16::from_le_bytes(self.slice(addr, 2)?.try_into().unwrap()))
    }

    pub fn write_u16(&mut self, addr: u32, val: u16) -> Result<(), ErrorType> {
        self.slice_mut(addr, 2)?.copy_from_slice(&val.to_le_bytes());
        Ok(())
    }

    pub fn read_u8(&self, addr: u32) -> Result<u8, ErrorType> {
        Ok(self.slice(addr, 1)?[0])
    }

    pub fn write_u8(&mut self, addr: u32, val: u8) -> Result<(), ErrorType> {
        self.slice_mut(addr, 1)?[0] = val;
        Ok(())
    }

    pub fn read_bytes(&self, addr: u32, len: u32) -> Result<&[u8], ErrorType> {
        self.slice(addr, len as usize)
    }

    pub fn write_bytes(&mut self, addr: u32, val: &[u8]) -> Result<(), ErrorType> {
        self.slice_mut(addr, val.len())?.copy_from_slice(val);
        Ok(())
    }
}
