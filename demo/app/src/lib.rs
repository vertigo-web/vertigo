#![deny(rust_2018_idioms)]
use vertigo::start_app;

mod app;

#[no_mangle]
pub fn start_application() {
    let component = app::State::component();
    start_app(component);
}
