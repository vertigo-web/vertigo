#![deny(rust_2018_idioms)]

use vertigo::{main, DomNode};
mod app;

#[main]
fn render() -> DomNode {
    let state = app::State::new();
    state.render()
}
