#![deny(rust_2018_idioms)]

use vertigo::{get_driver, DomNode};

mod app;
use crate::app::{render, State};

#[vertigo::main]
fn demo() -> DomNode {
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

    let state = State::new(ws_chat);
    render(&state)
}
