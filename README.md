Vertigo - reactive webassembly
===================

Features
--------------

* **Reactive dependencies** - A graph of values and clients (micro-subscriptions) that can automatically compute what to refresh after one value change
* **Real DOM** - No intermediate Virtual DOM mechanism is necessary
* **HTML/CSS macros** - Allows to construct Real DOM nodes using HTML and CSS
* **Server-side rendering** - Out of the box when using `vertigo-cli`

See [Changelog](https://github.com/vertigo-web/vertigo/blob/master/CHANGES.md) for recent features.

Go to **[TUTORIAL](https://github.com/vertigo-web/vertigo/blob/master/tutorial.md)** if you want to try.

Examples
--------------

Dependencies:

```toml
vertigo = "0.2.0-alpha"
```

Example 1:

```rust
use vertigo::{bind, dom, DomElement, start_app, Value};

pub fn app(count: &Value) -> DomElement {
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
                <p>"Counter: " { count }</p>
                <button on_click={decrement}>"-"</button>
                <button on_click={increment}>"+"</button>
            </body>
        </html>
    }
}

fn render() -> DomElement {
    let count = Value::new(0);
    app(&count)
}

#[no_mangle]
pub fn start_application() {
    start_app(render);
}
```

Example 2:

```rust
use vertigo::{css, dom, DomElement, start_app, Value, component};

#[component]
fn MyMessage(message: Value<String>) -> DomElement {
    dom! {
        <p>
            "Message to the world: "
            { self.message }
        </p>
    }
}

fn app(message: &Value) -> DomElement {
    let wrapper_css = css!("
        color: darkblue;
    ");

    dom! {
        <html>
            <head/>
            <body>
                <div css={wrapper_css}>
                    <MyMessage message={message} />
                </div>
            </body>
        </html>
    }
}

fn render() -> DomElement {
    let message = Value::new("Hello world!".to_string());
    app(&message)
}

#[no_mangle]
pub fn start_application() {
    start_app(render);
}
```

Take a look at **[More examples here](https://github.com/vertigo-web/vertigo/tree/master/examples)**.

Demo App - installation and usage
--------------

Make sure you're using nightly version of rust:

* `rustup default nightly`

Install cargo-make that takes care of all other dependencies:

* `cargo install cargo-make vertigo`

Build and run project using:

* `cargo make demo-start`

Eventually terminal will let you know that app is available under `http://localhost:4444/`

If you want to play around with the code, you can make cargo to watch for your changes:

* `cargo make demo-watch`

The browser will automatically refresh after the project has been recompiled.

--------------

To run the examples in watch mode (they will run on localhost:4444):
`cargo make examples-counter` or `cargo make examples-router` or `cargo make examples-trafficlights`

A community, soon to grow
--------------

* Discord server: <https://discord.gg/HAXtTeFrAf>
