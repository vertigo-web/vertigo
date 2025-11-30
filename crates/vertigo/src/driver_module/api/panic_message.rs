use crate::{dev::LongPtr, external_api::safe_wrappers::safe_panic_message};

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

        safe_panic_message(LongPtr::new(ptr, offser));
    }
}

use vertigo_macro::store;

#[store]
pub fn api_panic_message() -> PanicMessage {
    PanicMessage::new()
}
