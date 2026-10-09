pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Self { data: vec![0; size] }
    }

    pub fn read_u32(&self, addr: u32) -> u32 {
        let a = addr as usize;
        u32::from_le_bytes(self.data[a..a + 4].try_into().unwrap())
    }

    pub fn write_u32(&mut self, addr: u32, val: u32) {
        let a = addr as usize;
        self.data[a..a + 4].copy_from_slice(&val.to_le_bytes());
    }
}