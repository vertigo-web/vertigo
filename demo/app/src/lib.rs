#![deny(rust_2018_idioms)]

use vertigo::{main, DomNode, get_driver};
mod app;

#[main]
fn render() -> DomNode {
    let ws_chat = get_driver().env("ws_chat");
    let ws_chat = match ws_chat.as_deref() {
        Some("off") => None,
        _ => ws_chat,
    };

    get_driver().plains(|url| {
        if url == "/robots.txt" {
            Some("User-Agent: *\nDisallow: /search".to_string())
        } else {
            None
        }
    });

    let state = app::State::new(ws_chat);
    state.render()
}
