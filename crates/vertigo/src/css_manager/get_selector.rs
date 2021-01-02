use alloc::{
    string::String,
    format
};

pub fn get_selector(id: &u64) -> String {
    format!("autocss_{}", id)
}
