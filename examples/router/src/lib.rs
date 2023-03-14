#![deny(rust_2018_idioms)]
use vertigo::{start_app, DomElement};

mod app;

fn render() -> DomElement {
    let state = app::App::new();
    state.render()
}

#[no_mangle]
pub fn start_application() {
    start_app(render);
}
