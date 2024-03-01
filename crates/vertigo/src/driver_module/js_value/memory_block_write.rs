use super::memory_block::MemoryBlock;

#[derive(Debug)]
pub struct MemoryBlockWrite {
    block: MemoryBlock,
    offset: u32,
}

impl MemoryBlockWrite {
    pub fn new(block: MemoryBlock) -> MemoryBlockWrite {
        MemoryBlockWrite { block, offset: 0 }
    }

    pub fn get_block(self) -> MemoryBlock {
        self.block
    }

    pub fn write(&mut self, data: &[u8]) {
        self.block.write(self.offset, data);
        self.offset += data.len() as u32;
    }

    pub fn write_u8(&mut self, value: impl Into<u8>) {
        let value = value.into();
        self.write(&[value]);
    }

    pub fn write_u64(&mut self, value: u64) {
        let bytes: [u8; 8] = value.to_be_bytes();
        self.write(&bytes);
    }

    pub fn write_i64(&mut self, value: i64) {
        let bytes: [u8; 8] = value.to_be_bytes();
        self.write(&bytes);
    }

    pub fn write_u32(&mut self, value: u32) {
        let bytes: [u8; 4] = value.to_be_bytes();
        self.write(&bytes);
    }

    pub fn write_i32(&mut self, value: i32) {
        let bytes: [u8; 4] = value.to_be_bytes();
        self.write(&bytes);
    }

    pub fn write_u16(&mut self, value: u16) {
        let bytes: [u8; 2] = value.to_be_bytes();
        self.write(&bytes);
    }

    pub fn write_f64(&mut self, value: f64) {
        let bytes: [u8; 8] = value.to_be_bytes();
        self.write(&bytes);
    }
}
