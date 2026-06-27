#![deny(rust_2018_idioms)]

use vertigo::{DomNode, get_driver};

mod app;
use crate::app::{State, render};

#[vertigo::main]
fn demo() -> DomNode {
    let ws_chat = get_driver().env("ws_chat");
    let ws_chat = match ws_chat.as_deref() {
        Some("off") => None,
        _ => ws_chat,
    };

    let ws_collection = get_driver().env("ws_collection");
    let ws_collection = match ws_collection.as_deref() {
        Some("off") => None,
        _ => ws_collection,
    };

    get_driver().plains(|url| {
        if url == "/robots.txt" {
            Some("User-Agent: *\nDisallow: /search".to_string())
        } else {
            None
        }
    });

    let state = State::new(ws_chat, ws_collection);
    render(&state)
}
