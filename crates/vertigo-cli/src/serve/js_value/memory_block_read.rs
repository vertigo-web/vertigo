use super::memory_block::MemoryBlock;

#[derive(Debug)]
pub struct MemoryBlockRead {
    block: Vec<u8>,
    offset: u32,
}

impl MemoryBlockRead {
    pub fn new(buffer: MemoryBlock) -> MemoryBlockRead {
        MemoryBlockRead {
            block: buffer.convert_to_vec(),
            offset: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.block.len()
    }

    pub fn debug(&self) {
        log::info!("MemoryBlockRead DEBUG = {:02X?}", self.block);
    }

    pub fn debug_show_rest(&self) {
        let rest = &self.block[self.offset as usize..];
        log::info!("MemoryBlockRead DEBUG REST = {:02X?}", rest);
    }

    pub fn get_byte(&mut self) -> u8 {
        let result = self.block[self.offset as usize];
        self.offset += 1;
        result
    }

    pub fn get_u16(&mut self) -> u16 {
        let bytes: [u8; 2] = [
            self.block[self.offset as usize],
            self.block[self.offset as usize + 1]
        ];

        self.offset += 2;
        u16::from_be_bytes(bytes)
    }

    pub fn get_u32(&mut self) -> u32 {
        let bytes: [u8; 4] = [
            self.block[self.offset as usize],
            self.block[self.offset as usize + 1],
            self.block[self.offset as usize + 2],
            self.block[self.offset as usize + 3],
        ];

        self.offset += 4;
        u32::from_be_bytes(bytes)
    }

    pub fn get_i32(&mut self) -> i32 {
        let bytes: [u8; 4] = [
            self.block[self.offset as usize],
            self.block[self.offset as usize + 1],
            self.block[self.offset as usize + 2],
            self.block[self.offset as usize + 3],
        ];

        self.offset += 4;
        i32::from_be_bytes(bytes)
    }

    pub fn get_u64(&mut self) -> u64 {
        let bytes: [u8; 8] = [
            self.block[self.offset as usize],
            self.block[self.offset as usize + 1],
            self.block[self.offset as usize + 2],
            self.block[self.offset as usize + 3],
            self.block[self.offset as usize + 4],
            self.block[self.offset as usize + 5],
            self.block[self.offset as usize + 6],
            self.block[self.offset as usize + 7],
        ];

        self.offset += 8;
        u64::from_be_bytes(bytes)
    }

    pub fn get_f64(&mut self) -> f64 {
        let bytes: [u8; 8] = [
            self.block[self.offset as usize],
            self.block[self.offset as usize + 1],
            self.block[self.offset as usize + 2],
            self.block[self.offset as usize + 3],
            self.block[self.offset as usize + 4],
            self.block[self.offset as usize + 5],
            self.block[self.offset as usize + 6],
            self.block[self.offset as usize + 7],
        ];

        self.offset += 8;
        f64::from_be_bytes(bytes)
    }

    pub fn get_i64(&mut self) -> i64 {
        let bytes: [u8; 8] = [
            self.block[self.offset as usize],
            self.block[self.offset as usize + 1],
            self.block[self.offset as usize + 2],
            self.block[self.offset as usize + 3],
            self.block[self.offset as usize + 4],
            self.block[self.offset as usize + 5],
            self.block[self.offset as usize + 6],
            self.block[self.offset as usize + 7],
        ];

        self.offset += 8;
        i64::from_be_bytes(bytes)
    }

    pub fn get_string(&mut self, len: u32) -> Result<String, String> {
        let vec = self.get_vec(len);
        String::from_utf8(vec).map_err(|_| {
            String::from("String decoding error")
        })
    }

    pub fn get_vec(&mut self, len: u32) -> Vec<u8> {
        let from = self.offset as usize;
        let to = (self.offset + len) as usize;
        let slice = &self.block[from..to];

        self.offset += len;

        slice.to_vec()
    }

}
