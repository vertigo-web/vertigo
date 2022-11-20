use std::rc::Rc;

use crate::{driver_module::js_value::MemoryBlock, JsValue, struct_mut::HashMapMut};

#[derive(Clone, Default)]
pub struct Arguments {
    blocks: Rc<HashMapMut<u32, MemoryBlock>>
}

impl Arguments {
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

    pub fn get_by_ptr(&self, ptr: u32) -> JsValue {
        if ptr == 0 {
            return JsValue::Undefined;
        }

        let param = self.blocks.remove(&ptr);

        if let Some(param) = param {
            match JsValue::from_block(param) {
                Ok(value) => value,
                Err(err) => {
                    log::error!("get_by_ptr - error decode: {err}");
                    JsValue::Undefined
                }
            }
        } else {
            log::error!("get_by_ptr - not found MemoryBlock ptr={ptr}");
            JsValue::Undefined
        }
    }

    pub fn set(&self, memory_block: MemoryBlock) {
        let ptr = memory_block.get_ptr();
        self.blocks.insert(ptr, memory_block);
    }
}

