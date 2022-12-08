use std::time::{SystemTime, UNIX_EPOCH, Duration};

pub fn get_now() -> Duration {
    let start = SystemTime::now();
    start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
}
