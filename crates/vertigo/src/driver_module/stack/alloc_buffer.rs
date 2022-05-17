use super::{alloc::{MemoryBlock, WasmMemoryPtr, WasmMemorySize}, param_list::ParamItemNumber};

#[derive(Debug)]
pub struct AllocBuffer {
    block: MemoryBlock,
    offset: u32,
}

impl AllocBuffer {
    pub fn new(size: u32) -> AllocBuffer {
        let block = MemoryBlock::new(size);

        AllocBuffer {
            block,
            offset: 0,
        }
    }

    pub fn get_ptr(&self) -> WasmMemoryPtr {
        self.block.get_ptr()
    }

    pub fn get_size(&self) -> WasmMemorySize {
        self.block.get_size()
    }

    fn write(&mut self, data: &[u8]) {
        self.block.write(self.offset, data);
        self.offset += data.len() as u32;
    }

    pub fn write_u8(&mut self, value: u8) {
        self.write(&[value]);
    }

    pub fn write_param_type(&mut self, param_type: ParamItemNumber) {
        let type_u8 = param_type as u8;
        self.write_u8(type_u8);
    }

    pub fn write_mem_ptr(&mut self, ptr: WasmMemoryPtr) {
        let ptr = ptr.to_u32();
        self.write_u32(ptr);
    }

    pub fn write_mem_size(&mut self, size: WasmMemorySize) {
        let size = size.convert_to_u32();
        self.write_u32(size);
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

    pub fn convert_to_vec(self) -> Vec<u8> {
        self.block.convert_to_vec()
    }
}

