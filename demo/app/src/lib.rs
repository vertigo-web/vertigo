#![deny(rust_2018_idioms)]

use vertigo::start_app_fn;
mod app;

#[no_mangle]
pub fn start_application() {
    start_app_fn(|| {
        let state = app::State::new();
        let view = state.render();
        (state, view)
    });
}
