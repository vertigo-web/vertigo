#![deny(rust_2018_idioms)]

use vertigo::{main, DomNode, get_driver, dom};
mod app;

#[main]
fn render() -> DomNode {
    let Some(ws_chat) = get_driver().env("ws_chat") else {
        return dom! {
            <div>
                "The ws_chat variable env is missing"
            </div>
        }
    };

    let state = app::State::new(ws_chat);
    state.render()
}
