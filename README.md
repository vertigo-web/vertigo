[![Crates.io](https://img.shields.io/crates/d/vertigo.svg)](https://crates.io/crates/vertigo)
[![Crates.io](https://img.shields.io/crates/v/vertigo.svg)](https://crates.io/crates/vertigo)
[![Released API docs](https://docs.rs/vertigo/badge.svg)](https://docs.rs/vertigo)

Vertigo - reactive webassembly

## Features

* **Reactive dependencies** - A graph of values and clients (micro-subscriptions) that can automatically compute what to refresh after one value change
* **Real DOM** - No intermediate Virtual DOM mechanism is necessary
* **HTML/CSS macros** - Allows to construct Real DOM nodes using HTML and CSS
* **Server-side rendering** - Out of the box when using `vertigo-cli`

See [Changelog](https://github.com/vertigo-web/vertigo/blob/master/CHANGES.md) for recent features.

Go to **[TUTORIAL](https://github.com/vertigo-web/vertigo/blob/master/tutorial.md)** if you want to try.

## Installation

```sh
cargo install vertigo --version 0.2.0-alpha
```

## Examples

Dependencies:

```toml
vertigo = "0.2.0-alpha"
```

Example 1:

```rust
use vertigo::{dom, DomElement, Value, bind, start_app};

pub fn app() -> DomElement {
    let count = Value::new(0);

    let increment = bind!(count, || {
        count.change(|value| {
            *value += 1;
        });
    });

    let decrement = bind!(count, || {
        count.change(|value| {
            *value -= 1;
        });
    });

    dom! {
        <html>
            <head/>
            <body>
                <div>
                    <p>"Counter: " { count }</p>
                    <button on_click={decrement}>"-"</button>
                    <button on_click={increment}>"+"</button>
                </div>
            </body>
        </html>
    }
}

#[no_mangle]
pub fn start_application() {
    start_app(app);
}
```

Example 2:

```rust
use vertigo::{css, component, DomElement, Value, dom, start_app};

#[component]
pub fn MyMessage(message: Value<String>) -> DomElement {
    dom! {
        <p>
            "Message to the world: "
            { message }
        </p>
    }
}

fn app() -> DomElement {
    let message = Value::new("Hello world!".to_string());

    let main_div = css!("
        color: darkblue;
    ");

    dom! {
        <html>
            <head/>
            <body>
                <div css={main_div}>
                    <MyMessage message={message} />
                </div>
            </body>
        </html>
    }
}

#[no_mangle]
pub fn start_application() {
    start_app(app);
}
```

Take a look at **[More examples here](https://github.com/vertigo-web/vertigo/tree/master/examples)**.

## Demo App

### Prepare

Make sure you're using nightly version of rust:

* `rustup default nightly`

Install cargo-make and vertigo-cli:

* `cargo install cargo-make vertigo`

(Before stable version is released, append `--version 0.2.0-alpha` to install command)

### Run

Build and run project using:

* `cargo make demo-start`

Eventually terminal will let you know that app is available under `http://localhost:4444/`

If you want to play around with the code, you can make cargo to watch for your changes:

* `cargo make demo-watch`

The browser will automatically refresh after the project has been recompiled.

If you want to use "chat" in demo you need to first run websocket server in separate terminal with command:

* `cargo make demo-serve-api`

--------------

To run the examples in watch mode (they will run on localhost:4444):
`cargo make examples-counter` or `cargo make examples-router` or `cargo make examples-trafficlights`

## A community, soon to grow

* Discord server: <https://discord.gg/HAXtTeFrAf>
