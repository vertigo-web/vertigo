#![allow(dead_code)]
mod js_json_struct;
mod js_value_list_decoder;
mod js_value_struct;
mod memory_block;
mod memory_block_read;
mod memory_block_write;
mod serialize;

pub use js_json_struct::JsJson;
pub use js_value_struct::JsValue;
pub use memory_block::MemoryBlock;

#[cfg(test)]
mod tests;

pub use serialize::{from_json, JsJsonContext, JsJsonDeserialize};
