//https://docs.rs/web-sys/0.3.50/web_sys/struct.KeyboardEvent.html

use std::rc::Rc;

use crate::{struct_mut::ValueMut, JsValue};

/// Structure passed as a parameter to callback on on_key_down event.
#[derive(Clone, Debug, Default)]
pub struct ClickEvent {
    inner: Rc<ValueMut<ClickEventInner>>,
}

#[derive(Clone, Debug, Default)]
pub struct ClickEventInner {
    stop_propagation: bool,
    prevent_default: bool,
}

impl ClickEvent {
    pub fn stop_propagation(&self) {
        self.inner.change(|inner| inner.stop_propagation = true);
    }

    pub fn prevent_default(&self) {
        self.inner.change(|inner| inner.prevent_default = true);
    }
}

impl From<ClickEvent> for JsValue {
    fn from(val: ClickEvent) -> JsValue {
        let inner = val.inner.get();
        JsValue::Object([
            ("stop_propagation".to_string(), JsValue::from(inner.stop_propagation)),
            ("prevent_default".to_string(), JsValue::from(inner.prevent_default)),
        ].into_iter().collect())
    }
}

/// Structure passed as a parameter to callback on on_key_down event.
#[derive(Clone, Debug)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug)]
pub struct DropFileEvent {
    pub items: Vec<DropFileItem>,
}

impl DropFileEvent {
    pub fn new(items: Vec<DropFileItem>) -> DropFileEvent {
        DropFileEvent { items }
    }
}
