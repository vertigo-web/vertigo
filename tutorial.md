# Vertigo Tutorial

<!-- markdownlint-disable-next-line no-emphasis-as-heading -->
*Up to date with version 0.1.0-beta.2*

<!-- markdownlint-disable-next-line heading-increment -->
### Table of contents

1. [Prepare your system](#1-Prepare-your-system)
2. [Generate project](#2-Generate-project)
3. [First run](#3-First-run)
4. [Render](#4-Render)
5. [State](#5-State)
6. [Add new value](#6-Add-new-value)
7. [Set value](#7-Set-value)
8. [New component](#8-New-component)
9. [Add state to component](#9-Add-state-to-component)
10. [Input element](#10-Input-element)
11. [Computed value](#11-Computed-value)
12. [Parametrized styles](#12-Parametrized-styles)

## 1. Prepare your system

Vertigo uses trait aliases[^traitaliases] so until it stabilizes we need rust nightly to use vertigo. The easiest way to install rust nightly is to use [rustup](https://rustup.rs/). To install nightly toolchain and switch to it, run:

- `rustup default nightly`

If you're just starting with rust, make sure you have the essential tools for compiling programs in your system. We will also use a kickstarter template to start as fast as possible. This will require `openssl` installed in your system. For example, in debian-based distros this requires to have the following packages installed: `build-essential pkg-config libssl-dev`.

Now let's install the necessary tools to use the template:

- `cargo install cargo-generate cargo-make`

## 2. Generate project

Generate project in subdirectory using command:

- `cargo generate --name my-vertigo-app https://github.com/vertigo-web/vertigo-app-template`

## 3. First run

Enter the subdirectory:

- `cd my-vertigo-app`

Aside from the eusual `Cargo.toml` file and `src` dir there is `Makefile.toml` in the top directory of your project. There are tasks defined there to easily build and run the project. The most common task is:

- `cargo make watch`

which compiles the project, starts it and then waits for changes[^watch].

Tasks are configured in such way that all requirements will be installed automatically, that is `wasm32-unknown-unknown` target together with `cargo watch`, `wasm-pack` and `basic-http-server` packages.

After the task is completed you can point your browser to `http://127.0.0.1:3000/`[^bind] to see the "Hello World" message.

## 4. Render

Open `/src/app.rs` file.

```rust
use vertigo::{Computed, VDomElement, html, css_fn};

use super::state::State;

css_fn! { main_div, "
    color: darkblue;
" }

pub fn render(app_state: &Computed<State>) -> VDomElement {
    let state = app_state.get_value();

    html! {
        <div css={main_div()}>
            "Message to the world: "
            {state.message.get_value()}
        </div>
    }
}
```

This is the main (and the only for now) component. Component is just a `render` function that takes some `state` as a parameter.

Vertigo app mainly consists of three parts[^simplification]:

- *Dependency graph* - which holds the current state of app and triggers its leaf clients upon some change,
- *VirtualDOM elements* - that can be deps graph's clients and know how to update real DOM,
- *HTML macro* - which provides a convenient way to create VDOM elements.

If we want to be a little more detailed in this description, then it would be:

- Dependency graph holds values, computed values (computeds) and clients (render functions).
- Upon changing some value all dependent computeds get computed, and all dependent clients get rendered.
- Render function (a component) takes a computed state provided by the graph and returns a rendered element (`VDomElement`).
- Upon change in VDOM the real DOM is also updated.
- Components can provide the DOM with functions that get fired on events like `on_click`, which may modify the state, thus triggering necessary computing once again.

Now let's breakdown the code line by line:

```rust
use vertigo::{Computed, VDomElement, html, css_fn};
```

Here we import `Computed` and `VDomElement` structs that will define input and output of our render function.
We also import:

- `html!` macro to use HTML tags to define the shape of the resultant element, and
- `css_fn!` macro that helps define styles for DOM nodes using CSS syntax.

```rust
use super::state::State;
```

The component will be rendered using this `State` struct as the input value.

```rust
    css_fn! { main_div, "
        color: darkblue;
    " }
```

Using `css_fn!` macro we define here a function named `main_div` which returns styles[^styles] defined by `color: darkblue` body.

```rust
    pub fn render(app_state: &Computed<State>) -> VDomElement {
```

Here we define the render function itself.

```rust
    let state = app_state.get_value();
```

We need to get a direct reference to the state to be able to read its fields. This is done by `get_value()`[^subscription] method invoked on `Computed`.

```rust
    html! {
```

The `html!` macro always returns `VDomElement` object so it usually is at the end of the render function which returns the same type. You may as well pre-generate parts of the component using this macro and use it in the body of another `html!` invocation.

```rust
        <div css={main_div()}>
```

Here we define a VDom node using `div` tag, and assign it style using the css function `main_div`.

```rust
            "Message to the world: "
```

Next, in the `div` we insert a text node. Strings in `html!` macro must always be double-quoted. This assures us we won't miss a space between the text and the next VDom element.

```rust
            {state.message.get_value()}
```

Here we're inserting some value from the state. The `message` field in the state is of type `Value`. This type is similar to computed (has `get_value()` method), but it can also be changed using corresponding `set_value()` (more on this later).

```rust
        </div>
```

The `div` tag must be of course closed as in regular HTML.

## 5. State

Take a look at the state of the app in file `src/state.rs`. First let's see what is in the struct:

```rust
#[derive(PartialEq)]
pub struct State {
    driver: Driver,

    pub message: Value<String>,
}
```

I our state we have a `Driver` handle, which is our connection to two things:

- rendering output, usually a web browser,
- dependency graph, so we can create new reactive values.

We also have one `Value` with a string inside. The state and all types wrapped in `Value` are required to implement `PartialEq` so the dependency graph knows that values are changing.

To create our state we use `new()` method with gets a `Driver` handle, and returns a `Computed<State>`. Driver handle is used to create all necessary values and also to create the "computed" version of state itself.

```rust
impl State {
    pub fn new(driver: &Driver) -> Computed<State> {
        let state = State {
            driver: driver.clone(),
            message: driver.new_value("Hello world".to_string()),
        };

        driver.new_computed_from(state)
    }
}
```

To see how all these are connected, see `src/lib.rs`:

```rust
#[wasm_bindgen_derive(start)]
pub async fn start_application() {
    // Create rendering driver with dependency graph inside
    let driver = DriverBrowser::new();

    // Create application state lain on this graph
    let app_state = state::State::new(&driver);

    // Run application (using provided driver, state and render function) in browser rendering driver
    start_browser_app(driver, state::State::new, app::render).await;
}
```

## 6. Add new value

For starters let's add a new boolean value to the state and use it to render the component conditionally. Add

```rust
    pub strong: Value<bool>,
```

to State and

```rust
    strong: driver.new_value(true)
```

to `new()` method. Then in render function you can use this value:

```rust
pub fn render(app_state: &Computed<State>) -> VDomElement {
    let state = app_state.get_value();
    let message = state.message.get_value();

    let message_element = if *state.strong.get_value() {
        html! { <strong>{message}</strong> }
    } else {
        html! { <span>{message}</span> }
    };

    html! {
        <div css={main_div()}>
            "Message to the world: "
            {message_element}
        </div>
    }
}
```

In the browser the message should be now in bold.

## 7. Set value

Let's do some reactivity already. Add switch closure to our render function and use it in `html!` macro:

```rust
    let switch = move || {
        state.strong.set_value(
            !*state.strong.get_value()
        )
    };

    html! {
        <div css={main_div()}>
            "Message to the world: "
            {message_element}
            <button on_click={switch}>"Switch"</button>
        </div>
    }
```

We're using asterisk (`*`) on `.get_value()` to get out of `Rc`. Make sure you don't modify the state during rendering. If you do so, *vertigo* will tell you about it only in runtime.

## 8. New component

No app should be written as one big render function. Here how we can add a component to our app. Create file `src/list.rs`:

```rust
use vertigo::{VDomElement, html};

pub fn render() -> VDomElement {
    html! {
        <div>
            <p>"My list"</p>
            <ol>
                <li>"Item 1"</li>
                <li>"Item 2"</li>
            </ol>
        </div>
    }
}
```

Add to `/src/lib.rs`:

```rust
mod list;
```

And use it in main component in `src/app.rs`:

```rust
use crate::list;
```

(...)

```rust
    html! {
        <div css={main_div()}>
            "Message to the world: "
            {message_element}
            <button on_click={switch}>"Switch"</button>
            {list::render()}
        </div>
    }
```

## 9. Add state to component

For now our component just shows a static list which is not a usual way of rendering lists.
To go dynamic, add a struct to `src/list.rs`, which will be our sub-state for the component:

```rust
use vertigo::{Computed, Driver, Value, VDomElement, html};

#[derive(PartialEq)]
pub struct State {
    items: Value<Vec<String>>,
}

impl State {
    pub fn new(driver: &Driver) -> Computed<State> {
        let state = State {
            items: driver.new_value(vec![
                "Item 1".to_string(),
                "Item 2".to_string(),
            ]),
        };

        driver.new_computed_from(state)
    }
}
```

And add this sub-state into our main state in `src/state.rs`:

```rust
use crate::list;

#[derive(PartialEq)]
pub struct State {
    driver: Driver,

    pub message: Value<String>,
    pub strong: Value<bool>,
    pub list: Computed<list::State>,
}

impl State {
    pub fn new(driver: &Driver) -> Computed<State> {
        let state = State {
            driver: driver.clone(),
            message: driver.new_value("Hello world".to_string()),
            strong: driver.new_value(true),
            list: list::State::new(driver),
        };

        driver.new_computed_from(state)
    }
}
```

Now we can use this state to render our component dynamically. In `src/list.rs` modify `render` function this way:

```rust
pub fn render(state: &Computed<State>) -> VDomElement {
    let state = state.get_value();
    let items = state.items.get_value();

    let elements = items.iter()
        .map(|item|
            html! {
                <li>{item}</li>
            }
        );

    html! {
        <div>
            <p>"My list"</p>
            <ol>
                { ..elements }
            </ol>
        </div>
    }
}
```

As you can see the function now takes its state as a parameter, gets items out of this state and maps them into a vector of `<li>` elements. The vector can then be inserted as a list of children in `html!` macro using `..elements` notation.

Now `html!` macro in our main `src/app.rs` yields an error - we need to provide a state for `list::render` function:

```rust
    let list_state = &state.list;

    html! {
        <div css={main_div()}>
            "Message to the world: "
            {message_element}
            <button on_click={switch}>"Switch"</button>
            {list::render(&list_state)}
        </div>
    }
```

Another error appears:

```text
borrow of moved value: `state`
borrow occurs due to deref coercion to `state::State`
```

This is because of the fact that our `switch` closure takes the whole state. Happily enough `Computed` and `Value` can be shallow-cloned, so make the `switch` closure look like this:

```rust
    let switch = {
        let strong = state.strong.clone();
        move || {
            strong.set_value(
                !*strong.get_value()
            )
        }
    };
```

This is a common pattern for creating event handlers in *vertigo*.

## 10. Input element

Our component cries out for adding more items. To implement this we need to:

1. add input element and button next to it,
2. make value of this input be taken from the `Value` stored in the state,
3. make typing in this input `change` the value in the state,
4. upon clicking on the button a closure should be fired which will `add` the value as a new element in the list and erase input value.

So the whole `src/list.rs` will look like this:

```rust
use vertigo::{Computed, Driver, Value, VDomElement, html};

#[derive(PartialEq)]
pub struct State {
    items: Value<Vec<String>>,
    new_item: Value<String>,
}

impl State {
    pub fn new(driver: &Driver) -> Computed<State> {
        let state = State {
            items: driver.new_value(vec![
                "Item 1".to_string(),
                "Item 2".to_string(),
            ]),
            new_item: driver.new_value("".to_string()),
        };

        driver.new_computed_from(state)
    }

    pub fn add(&self) -> impl Fn() {
        let items = self.items.clone();
        let new_item = self.new_item.clone();
        move || {
            let mut items_vec = items.get_value().to_vec();
            items_vec.push(new_item.get_value().to_string());
            items.set_value(items_vec);
            new_item.set_value("".to_string());
        }
    }

    pub fn change(&self) -> impl Fn(String) {
        let new_item = self.new_item.clone();
        move |value: String| {
            new_item.set_value(value)
        }
    }
}

pub fn render(state: &Computed<State>) -> VDomElement {
    let state = state.get_value();
    let items = state.items.get_value();

    let elements = items.iter()
        .map(|item|
            html! {
                <li>{item}</li>
            }
        );

    let new_value = &*state.new_item.get_value();

    html! {
        <div>
            <p>"My list"</p>
            <ol>
                { ..elements }
            </ol>
            <input value={new_value} on_input={state.change()} />
            <button on_click={state.add()}>"Add"</button>
        </div>
    }
}
```

We've added 2 methods to state, both returning an event handler. Method `add` returns a handler for the `on_click` event so it's a bare `Fn()` without any argument. Method `change` returns a handler for the `on_input` event so it accepts a `String` value. Both methods clone necessary values to have them moved into the closure returned as event handler.

## 11. Computed value

It is possible to have a value that is automatically computed. Let's show the amount of items in the list. First add a computed type to the list's state:

```rust
#[derive(PartialEq)]
pub struct State {
    items: Value<Vec<String>>,
    new_item: Value<String>,
    count: Computed<usize>,
}
```

Then we need to reorganize a little how we create an instance of the state:

```rust
    pub fn new(driver: &Driver) -> Computed<State> {
        let items = driver.new_value(vec![
            "Item 1".to_string(),
            "Item 2".to_string(),
        ]);

        let count = {
            let items = items.clone();
            driver.from(move || items.get_value().len())
        };

        let state = State {
            items,
            new_item: driver.new_value("".to_string()),
            count,
        };

        driver.new_computed_from(state)
    }
```

First we need to create the list of items, then we will create the `Computed` using the `Driver::from` method which accepts a function that calculates the value. We need to clone "the access"[^clone] to the list first to be able to move it into the closure. As it was stated earlier, firing `.get_value()` method creates a dependency in the driver's graph, so every client reading computed will get a new value from the computed everytime the list has changed.

Now we can use this computed in render function:

```rust
    let count = *state.count.get_value();

    html! {
        <div>
            <p>"My list (" { count } ")"</p>
            (...)
```

## 12. Parametrized styles

As a bonus feature, we'll delve inthe styles. First we'll make the list to change font color for every other row. Remember to import `css_fn` from vertigo.

```rust
css_fn! { alternate_rows, "
    color: black;

    :nth-child(odd) {
        color: blue;
    };
" }
```

And use these styles in `html!` macro:

```rust
            html! {
                <li css={alternate_rows()}>{item}</li>
            }
```

Now we want to have particular items emphasized by different background. Let's say all items ending with an exclamation mark. To create a parameterized css function we need to drop usage of the `css_fn` macro, and create the function ourselves. So instead of `css_fn` we need to import the `css` macro and the `Css` type, which *vertigo* uses to define a group of css rules.

```rust
fn alternate_rows(excl: bool) -> Css {
    let bg_color = if excl { "yellow" } else { "inherit" };

    css!("
        color: black;
        background: { bg_color };

        :nth-child(odd) {
            color: blue;
        };
    ")
}
```

And here's the usage in render:

```rust
    let elements = items.iter()
        .map(|item| {
            let excl = item.ends_with('!');
            html! {
                <li css={alternate_rows(excl)}>{item}</li>
            }
        });
```

## Further reading

Complete code for this tutorial should be found [here](https://github.com/vertigo-web/vertigo-app-template/tree/tutorial).

For any more complex scenarios please refer to the examples in the [demo](/demo/src/app) package.

[^traitaliases]: https://github.com/rust-lang/rust/issues/41517

[^watch]: You still need to refresh the page in the browser after making changes and after project rebuilds.

[^bind]: If you want to enter your app from outside your local machine then in `Makefile.toml` in section `[tasks.serve]` change `127.0.0.1` to `0.0.0.0`.

[^simplification]: This is a shameful simplification but enough for a tutorial - the correct description will be able to be found in future more robust documentation.

[^styles]: Styles are being attached to document's `HEAD` as classes with unique auto-generated names. These names are then used in HTML tags. This way you can use such CSS functions multiple times to different HTML tags and they'll all use the same class.

[^subscription]: `get_value()` method creates a subscription in dependency graph so the render function is now dependent on the value, and will be fired everytime the value changes. This is similar to how the MobX library works in React world.

[^clone]: Every `Value` and `Computed` wrap it's inner value in an `Rc` so cloning does not clone the content. It just creates another pointer - a handler to access the value.
