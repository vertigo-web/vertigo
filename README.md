Vertigo - reactive webassembly
===================

Features
--------------

* **Virtual DOM** - Lightweight representation of JavaScript DOM that can be used to optimally update real DOM
* **Reactive dependencies** - A graph of values and clients that can automatically compute what to refresh after one value change
* **HTML/CSS macros** - Allows to construct Virtual DOM nodes using HTML and CSS

See [Changelog](/CHANGES.md) for recent features.

Go to **[TUTORIAL](/tutorial.md)** if you want to try.

Example
--------------

Dependencies:

```toml
vertigo = "0.1.0-beta.4"
```

Code:

```rust
use vertigo::{html, VDomElement, VDomComponent, Value, bind, start_app};

#[derive(Clone)]
pub struct State {
    pub count: Value<i32>,
}

impl State {
    pub fn new() -> State {
        State {
            count: Value::new(0),
        }
    }

    pub fn render(&self) -> VDomComponent {
        VDomComponent::from(self, render)
    }
}

pub fn render(state: &State) -> VDomElement {
    let increment = bind(state).call(|state| {
        state.count.set_value(*state.count.get_value() + 1);
    });

    let decrement = bind(state).call(|state| {
        state.count.set_value(*state.count.get_value() - 1);
    });

    html! {
        <div>
            <p>"Counter: " { *state.count.get_value() }</p>
            <button on_click={decrement}>"-"</button>
            <button on_click={increment}>"+"</button>
        </div>
    }
}

#[no_mangle]
pub fn start_application() {
    let state = State::new();
    let component = state.render();
    start_app(component);
}
```

Take a look at **[More examples here](/examples)**.

Demo App - installation and usage
--------------

Make sure you're using nigthly version of rust:

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
