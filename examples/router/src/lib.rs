#![deny(rust_2018_idioms)]
use vertigo::start_app;

mod app;

#[no_mangle]
pub fn start_application() {
    let state = app::App::new();
    let view = state.render();
    start_app(state, view);
}
