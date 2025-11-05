use super::external_api::api::safe_wrappers::{
    safe_panic_message as panic_message,
};

#[derive(Clone, Copy)]
pub struct PanicMessage {
}

impl PanicMessage {
    pub fn new() -> PanicMessage {
        PanicMessage { }
    }

    pub fn show(&self, message: impl Into<String>) {
        let message = message.into();
        let ptr = message.as_ptr() as u64;
        let len = message.len() as u64;
        panic_message((ptr << 32) + len);
    }
}


use vertigo_macro::store;

#[store]
pub fn api_panic_message() -> PanicMessage {
    PanicMessage::new()
}
