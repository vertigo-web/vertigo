use std::rc::Rc;

use crate::{driver_module::js_value::MemoryBlock, struct_mut::HashMapMut, JsValue, LongPtr};

#[derive(Clone)]
pub struct Arguments {
    blocks: Rc<HashMapMut<u32, MemoryBlock>>, //TODO - u64 niech będzie
}

impl Arguments {
    pub fn new() -> Arguments {
        Arguments {
            blocks: Rc::new(HashMapMut::new()),
        }
    }

    pub fn alloc(&self, size: u32) -> u32 {
        let block = MemoryBlock::new(size);
        let ptr = block.get_ptr();
        self.blocks.insert(ptr, block);
        ptr
    }

    pub fn free(&self, pointer: u32) {
        let block = self.blocks.remove(&pointer);

        if block.is_none() {
            log::error!("Failed to release memory block at address: {pointer}");
        }
    }

    pub fn get_by_long_ptr(&self, long_ptr: LongPtr) -> JsValue {
        if long_ptr.is_undefined() {
            return JsValue::Undefined;
        }

        let (ptr, _) = long_ptr.into_parts();

        let param = self.blocks.remove(&ptr);

        if let Some(param) = param {
            match JsValue::from_block(param) {
                Ok(value) => value,
                Err(err) => {
                    panic!("get_by_ptr - error decode: {err}");
                }
            }
        } else {
            panic!("get_by_ptr - not found MemoryBlock ptr={ptr}");
        }
    }

    pub fn set(&self, memory_block: MemoryBlock) {
        let ptr = memory_block.get_ptr();
        self.blocks.insert(ptr, memory_block);
    }
}

use vertigo_macro::store;

#[store]
pub fn api_arguments() -> Arguments {
    Arguments::new()
}
