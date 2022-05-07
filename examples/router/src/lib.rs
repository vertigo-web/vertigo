#![deny(rust_2018_idioms)]
use vertigo::start_app;

mod app;

#[no_mangle]
pub fn start_application() {
    start_app(app::State::component());
}
