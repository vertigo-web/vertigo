#![deny(rust_2018_idioms)]
use vertigo::{main, DomElement};

mod app;

#[main]
fn render() -> DomElement {
    let state = app::App::new();
    state.render()
}
