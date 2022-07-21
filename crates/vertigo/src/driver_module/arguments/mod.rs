use std::rc::Rc;

use crate::struct_mut::HashMapMut;

use self::{
    memory_block::MemoryBlock,
    params::{decode_params, ParamItem}
};

pub mod memory_block_write;
pub mod memory_block_read;
pub mod memory_block;
pub mod params;
pub mod param_builder;

#[derive(Clone)]
pub struct Arguments {
    blocks: Rc<HashMapMut<u32, MemoryBlock>>
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

    pub fn get_by_ptr(&self, ptr: u32) -> Option<ParamItem> {
        let param = self.blocks.remove(&ptr);

        if let Some(param) = param {
            match decode_params(param) {
                Ok(value) => Some(value),
                Err(err) => {
                    log::error!("get_by_ptr - error decode: {err}");
                    None
                }
            }
        } else {
            log::error!("get_by_ptr - not found MemoryBlock ptr={ptr}");
            None
        }
    }

    pub fn set(&self, memory_block: MemoryBlock) {
        let ptr = memory_block.get_ptr();
        self.blocks.insert(ptr, memory_block);
    }
}

