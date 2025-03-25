#![deny(rust_2018_idioms)]
#![feature(ergonomic_clones)]
#![allow(incomplete_features)]

use vertigo::{main, DomNode, get_driver, dom};
mod app;

#[main]
fn render() -> DomNode {
    let Some(ws_chat) = get_driver().env("ws_chat") else {
        get_driver().set_status(500);
        return dom! {
            <html>
                <body>
                    <div>
                        "The ws_chat variable env is missing"
                    </div>
                </body>
            </html>
        }
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
