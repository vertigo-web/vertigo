use std::rc::Rc;

use crate::{
    computed::struct_mut::HashMapMut, dev::LongPtr, driver_module::js_value::MemoryBlock, JsJson,
};

#[derive(Clone)]
pub struct Arguments {
    blocks: Rc<HashMapMut<LongPtr, MemoryBlock>>,
}

impl Arguments {
    pub fn new() -> Arguments {
        Arguments {
            blocks: Rc::new(HashMapMut::new()),
        }
    }

    pub fn alloc(&self, size: u32) -> LongPtr {
        let block = MemoryBlock::new(size);
        let long_ptr = block.get_ptr_long();

        self.blocks.insert(long_ptr, block);
        long_ptr
    }

    pub fn free(&self, pointer: LongPtr) {
        let block = self.blocks.remove(&pointer);

        if block.is_none() {
            log::error!(
                "Failed to release memory block at address: {}",
                pointer.get_long_ptr()
            );
        }
    }

    pub fn get_by_long_ptr(&self, long_ptr: LongPtr) -> JsJson {
        if long_ptr.is_undefined() {
            return JsJson::Null;
        }

        let param = self.blocks.remove(&long_ptr);

        if let Some(param) = param {
            match JsJson::from_block(param) {
                Ok(value) => value,
                Err(err) => {
                    panic!("get_by_ptr - error decode: {err}");
                }
            }
        } else {
            panic!(
                "get_by_ptr - not found MemoryBlock ptr={}",
                long_ptr.get_long_ptr()
            );
        }
    }

    pub fn set(&self, memory_block: MemoryBlock) {
        let ptr = memory_block.get_ptr_long();
        self.blocks.insert(ptr, memory_block);
    }

    #[allow(unused)]
    pub fn dump(&self, ptr: LongPtr) -> Option<Vec<u8>> {
        self.blocks.get_and_map(&ptr, |block| block.dump())
    }
}

use vertigo_macro::store;

#[store]
pub fn api_arguments() -> Arguments {
    Arguments::new()
}
