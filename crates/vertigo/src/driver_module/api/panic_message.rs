#[derive(Clone, Copy)]
pub struct PanicMessage {
    panic_message: fn(ptr: u32, size: u32),
}

impl PanicMessage {
    pub fn new(panic_message: fn(ptr: u32, size: u32)) -> PanicMessage {
        PanicMessage { panic_message }
    }

    pub fn show(&self, message: impl Into<String>) {
        let message = message.into();
        let ptr = message.as_ptr() as u32;
        let len = message.len() as u32;
        (self.panic_message)(ptr, len);
    }
}
