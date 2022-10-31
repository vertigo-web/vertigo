Vertigo - reactive webassembly
===================

Features
--------------

* **Reactive dependencies** - A graph of values and clients (micro-subscriptions) that can automatically compute what to refresh after one value change
* **Real DOM** - No intermediate Virtual DOM mechanism is necessary
* **HTML/CSS macros** - Allows to construct Real DOM nodes using HTML and CSS

See [Changelog](https://github.com/vertigo-web/vertigo/blob/master/CHANGES.md) for recent features.

Go to **[TUTORIAL](https://github.com/vertigo-web/vertigo/blob/master/tutorial.md)** if you want to try.

Examples
--------------

Dependencies:

```toml
vertigo = "0.1"
```

Example 1:

```rust
use vertigo::{bind, dom, DomElement, start_app, Value};

pub fn app() -> DomElement {
    let count = Value::new(0);

    let increment = bind!(|count| {
        count.change(|value| {
            *value += 1;
        });
    });

    let decrement = bind!(|count| {
        count.change(|value| {
            *value -= 1;
        });
    });

    dom! {
        <div>
            <p>"Counter: " { count }</p>
            <button on_click={decrement}>"-"</button>
            <button on_click={increment}>"+"</button>
        </div>
    }
}

#[no_mangle]
pub fn start_application() {
    start_app(app);
}
```

Example 2:

```rust
use vertigo::{css, dom, DomElement, start_app, Value};

pub struct MyMessage {
    pub message: Value<String>,
}

impl MyMessage {
    pub fn mount(self) -> DomElement {
        dom! {
            <p>
                "Message to the world: "
                { self.message }
            </p>
        }
    }
}

fn app() -> DomElement {
    let message = Value::new("Hello world!".to_string());

    let wrapper_css = css!("
        color: darkblue;
    ");

    dom! {
        <div css={wrapper_css}>
            <MyMessage message={message} />
        </div>
    }
}

#[no_mangle]
pub fn start_application() {
    start_app(app);
}
```

Take a look at **[More examples here](https://github.com/vertigo-web/vertigo/tree/master/examples)**.

Demo App - installation and usage
--------------

Make sure you're using nightly version of rust:

* `rustup default nightly`

Install cargo-make that takes care of all other dependencies:

* `cargo install cargo-make`

Build and run project using:

* `cargo make demo-start`

Eventually terminal will let you know that app is available under `http://localhost:3000/`

If you want to play around with the code, you can make cargo to watch for your changes:

* `cargo make demo-watch`

Keep in mind that you still need to refresh page in the browser after project recompiles.

To compile all examples run:

* `cargo make examples-build`

This will build examples in `examples/build` directory. Now point your browser to `index.html` file of a particular example.

A community, soon to grow
--------------

* Discord server: <https://discord.gg/HAXtTeFrAf>
