pub fn log_ok(message: impl Into<String>) {
    let message = message.into();
    println!("vertigo: {message}");
}

pub fn log_error(message: impl Into<String>) {
    let message = message.into();
    eprintln!("vertigo: error: {message}");
}
