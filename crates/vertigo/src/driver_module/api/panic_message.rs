use crate::LongPtr;

use super::external_api::api::safe_wrappers::safe_panic_message as panic_message;

#[derive(Clone, Copy)]
pub struct PanicMessage {}

impl PanicMessage {
    pub fn new() -> PanicMessage {
        PanicMessage {}
    }

    pub fn show(&self, message: impl Into<String>) {
        let message = message.into();
        let ptr = message.as_ptr() as u32;
        let offser = message.len() as u32;

        panic_message(LongPtr::new(ptr, offser));
    }
}

use vertigo_macro::store;

#[store]
pub fn api_panic_message() -> PanicMessage {
    PanicMessage::new()
}
