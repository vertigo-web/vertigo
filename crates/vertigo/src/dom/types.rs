
//https://docs.rs/web-sys/0.3.50/web_sys/struct.KeyboardEvent.html

use std::rc::Rc;

/// Structure passed as a parameter to callback on on_key_down event.
#[derive(Debug, Clone)]
pub struct KeyDownEvent {
    pub key: String,
    pub code: String,
    pub alt_key: bool,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub meta_key: bool,
}

impl std::fmt::Display for KeyDownEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KeyDownEvent={}", self.key)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropFileItem {
    pub name: String,
    pub data: Rc<Vec<u8>>,
}

impl DropFileItem {
    pub fn new(name: String, data: Vec<u8>) -> DropFileItem {
        DropFileItem {
            name,
            data: Rc::new(data),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DropFileEvent {
    pub items: Vec<DropFileItem>,
}

impl DropFileEvent {
    pub fn new(items: Vec<DropFileItem>) -> DropFileEvent {
        DropFileEvent {
            items
        }
    }
}
