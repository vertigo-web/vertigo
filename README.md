# Vertigo

A reactive Real-DOM library with SSR for Rust

[![crates.io](https://img.shields.io/crates/v/vertigo)](https://crates.io/crates/vertigo)
[![Documentation](https://docs.rs/vertigo/badge.svg)](https://docs.rs/vertigo)
![MIT or Apache 2.0 licensed](https://img.shields.io/crates/l/vertigo.svg)
[![Dependency Status](https://deps.rs/crate/vertigo/0.8.3/status.svg)](https://deps.rs/crate/vertigo/0.8.3)
[![CI](https://github.com/vertigo-web/vertigo/actions/workflows/pipeline.yaml/badge.svg)](https://github.com/vertigo-web/vertigo/actions/workflows/pipeline.yaml)
[![downloads](https://img.shields.io/crates/d/vertigo.svg)](https://crates.io/crates/vertigo)

## Features

* **Reactive dependencies** - A graph of values and clients (micro-subscriptions) that can automatically compute what to refresh after one value change
* **Real DOM** - No intermediate Virtual DOM mechanism is necessary
* **HTML/CSS macros** - Allows to construct Real DOM nodes using HTML and CSS
* **Server-side rendering** - Out of the box when using `vertigo-cli`

See [Changelog](https://github.com/vertigo-web/vertigo/blob/master/CHANGES.md) for recent features.

Go to **[TUTORIAL](https://github.com/vertigo-web/vertigo/blob/master/tutorial.md)** if you want to try.

For more information go to vertigo home website **[vertigo.znoj.pl](https://vertigo.znoj.pl/)**.

## Examples

Dependencies:

```toml
vertigo = "0.8"
```

Example 1:

```rust
use vertigo::{dom, DomNode, Value, store, main};

#[store]
fn state_count() -> Value<u32> {
    Value::new(0)
}

#[main]
pub fn app() -> DomNode {
    dom! {
        <html>
            <head/>
            <body>
                <p>"Counter: " { state_count().get() }</p>
                <button
                    on_click={|| {
                        let current = state_count();
                        current.set(current.get() - 1);
                    }}
                >
                    "-"
                </button>
                <button
                    on_click={|| {
                        let current = state_count();
                        current.set(current.get() + 1);
                    }}
                >
                    "+"
                </button>
            </body>
        </html>
    }
}
```

Example 2:

```rust
use vertigo::{css, component, DomNode, Value, dom, main};

#[component]
pub fn MyMessage(message: Value<String>) {
    dom! {
        <p>
            "Message to the world: "
            { message }
        </p>
    }
}

#[main]
fn app() -> DomNode {
    let message = Value::new("Hello world!".to_string());

    let main_div = css! {"
        color: darkblue;
    "};

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
```

Take a look at **[More examples here](https://github.com/vertigo-web/vertigo/tree/master/examples)**.

## Installation of `vertigo-cli` tool

To ease process or development use
[vertigo-cli](https://github.com/vertigo-web/vertigo/blob/master/crates/vertigo-cli) tool
that allows to _build_, _serve_ and _watch_ your project.

```sh
cargo install --force vertigo-cli
```

## Demo App

### Prepare

Make sure you're using nightly version of rust:

* `rustup default nightly`

Install cargo-make and vertigo-cli:

* `cargo install cargo-make vertigo-cli`

### Run

Build and run project using:

* `cargo make demo`

Eventually terminal will let you know that app is available under `http://localhost:4444/`

If you want to play around with the demo code, run:

* `cargo make demo-watch`

It should automatically recompile upon changes and the browser tab should be informed to refresh.
Note that this compiles the code in debug mode so the WASM is not optimized.

--------------

To run the examples in watch mode (they will run on localhost:4444):
`cargo make examples-counter` or `cargo make examples-router` or `cargo make examples-trafficlights`

## A community, soon to grow

* Discord server: <https://discord.gg/HAXtTeFrAf>
