use super::alloc::{MemoryBlock, WasmMemoryPtr, WasmMemorySize};

#[derive(Debug)]
pub struct AllocString {
    block: MemoryBlock,
}

impl AllocString {
    pub fn new(size: u32) -> AllocString {
        let block = MemoryBlock::new(size);
        AllocString {
            block
        }
    }

    pub fn get_ptr(&self) -> WasmMemoryPtr {
        self.block.get_ptr()
    }

    pub fn get_size(&self) -> WasmMemorySize {
        self.block.get_size()
    }

    pub fn convert_to_string(self) -> String {
        let data = self.block.convert_to_vec();
        String::from_utf8(data).unwrap()
    }
}

