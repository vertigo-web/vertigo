use proc_macro::{Span};

pub fn log_ok(message: impl Into<String>) {
    let message = message.into();
    println!("ok: {message}");
}

pub fn log_error(message: impl Into<String>) {
    let message = message.into();
    emit_warning!(Span::call_site(), "{}", message);
}
