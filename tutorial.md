# Vertigo Tutorial

<!-- markdownlint-disable-next-line no-emphasis-as-heading -->
*Up to date with version 0.1.0-beta.3*

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

Aside from the usual `Cargo.toml` file and `src` dir, there is `Makefile.toml` in the top directory of your project. There are tasks defined there to easily build and run the project. The most common task is:

- `cargo make watch`

which compiles the project, starts it and then waits for changes[^watch].

Tasks are configured in such way that most of requirements will be installed automatically, that is `wasm32-unknown-unknown` target together with `cargo watch` and `basic-http-server` packages.

Only `wasm-opt` needs to be installed manually.
Install `binaryen` package in your linux distribution (f. ex. `apt-get install binaryen` on Debian/Ubuntu) or go to [https://github.com/WebAssembly/binaryen/discussions/3797](https://github.com/WebAssembly/binaryen/discussions/3797) for other instructions.

After the task `watch` is completed you can point your browser to `http://127.0.0.1:3000/`[^bind] to see the "Hello World" message.

## 4. Render

Open `/src/render.rs` file.

```rust
use vertigo::{DomElement, dom, css_fn};

use super::state::State;

css_fn! { main_div, "
    color: darkblue;
" }

pub fn render(state: &State) -> DomElement {
    dom! {
        <div css={main_div()}>
            "Message to the world: "
            <text computed={state.message.to_computed()} />
        </div>
    }
}
```

This is the main (and the only for now) function that renders something. It transforms `State` into `DomElement`.

Vertigo app mainly consists of three parts[^simplification]:

- *Dependency graph* - which holds the current state of app and triggers its leaf clients upon some change,
- *DOM elements* - that can be deps graph's clients and know how to update itself on the page,
- *HTML macro* - which provides a convenient way to create VDOM elements.

If we want to be a little more detailed in this description, then it would be:

- Dependency graph holds values, computed values (computeds) and clients (render functions).
- Upon changing some value all dependent computeds get computed, and all dependent clients get rendered.
- Render function (a component) takes a computed state provided by the graph and returns a rendered element (`DomElement`).
- Upon any change in state, DOM is also updated if necessary.
- Components can provide the DOM with functions that get fired on events like `on_click`, which may modify the state, thus triggering necessary computing once again.

Now let's breakdown the code line by line:

```rust
use vertigo::{DomElement, dom, css_fn};
```

Here we import `DomElement` struct that will define output of our render function (a reactive component).
We also import:

- `dom!` macro to use HTML tags to define the shape of the resultant element, and
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
    pub fn render(state: &State) -> DomElement {
```

Here we define the render function itself.

```rust
    dom! {
```

The `dom!` macro always returns `DomElement` object so it usually is at the end of the render function which returns the same type. You may as well pre-generate parts of the component using this macro and use it in the body of another `dom!` invocation.

```rust
        <div css={main_div()}>
```

Here we define a DOM node using `div` tag, and assign it style using the css function `main_div`.

```rust
            "Message to the world: "
```

Next, in the `div` we insert a text node. Strings in `dom!` macro must always be double-quoted. This assures us we won't miss a space between the text and the next DOM element.

```rust
            <text computed={state.message.to_computed()} />
```

Here we're inserting some value from the state. The `message` field in the state is of type `Value`.
It is a read/write box that has a `to_computed()` method which transforms the value into a read-only observable for rendering.

```rust
        </div>
```

The `div` tag must be of course closed as in regular HTML.

## 5. State

Take a look at the state of the app in file `src/state.rs`. First let's see what is in the struct:

```rust
pub struct State {
    pub message: Value<String>,
}
```

I our state we have one `Value` with a string inside. The state and all types wrapped in `Value` are required to implement `PartialEq` so the dependency graph knows that values are changing.

To create a reactive component (`DomElement`) we first create it's state and then use the render function.

```rust
impl State {
    pub fn component() -> DomElement {
        let state = State {
            message: Value::new("Hello world".to_string()),
        };

        app::render(state)
    }
}
```

To see how all these are connected, see `src/lib.rs`:

```rust
#[no_mangle]
pub fn start_application() {
    start_app(state::State::component);
}
```

## 6. Add new value

For starters let's add a new boolean value to the state and use it to render the component conditionally. Add

```rust
    pub strong: Value<bool>,
```

to State and

```rust
    strong: Value::new(true)
```

to `State::component()` method. Then in render function you can use this value:

```rust
pub fn render(state: &State) -> DomElement {
    let message = state.message.clone();

    let message_element = state.strong.render_value(move |strong|
        if strong {
            dom! { <strong><text computed={message.to_computed()}/></strong> }
        } else {
            dom! { <span><text computed={message.to_computed()}/></span> }
        }
    );

    dom! {
        <div css={main_div()}>
            "Message to the world: "
            {message_element}
        </div>
    }
}
```

In the browser the message should be now in bold.

## 7. Set value

Let's do some reactivity already. Import `vertigo::bind`, add switch closure to our render function and use it in `dom!` macro:

```rust
    let switch = bind(&state.strong).call(|ctx, strong|
        strong.set(
            !strong.get(ctx)
        )
    );


    html! {
        <div css={main_div()}>
            "Message to the world: "
            {message_element}
            <button on_click={switch}>"Switch"</button>
        </div>
    }
```

To create an event handler in a handy way, vertigo introduces a "binding" mechanism. This reminds a `.bind()` function from JavaScript world, but the reason is different.
Binding a value automatically creates a clone of the value that can be used upon firing the event (that is, upon invoking `call()` method).
Happily enough, everything wrapped in a `Value<T>` have a shallow cloning implemented.
The `call()` method also provides `Context` which allows you to read the bound value in a responsive way (using `get(ctx)`[^subscription] method on a `Value`).

## 8. New component

No app should be written as one big render function. Here how we can add a component to our app. Create file `src/list.rs`:

```rust
use vertigo::{DomElement, dom};

pub fn render() -> DomElement {
    dom! {
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

And use it in main component in `src/render.rs`:

```rust
use crate::list;
```

(...)

```rust
    dom! {
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
To go dynamic, add a struct to `src/list.rs`, which will be our sub-state for the component, and make the render function a method of this sub-state.

```rust
use vertigo::{Value, DomElement, dom};

#[derive(Clone)]
pub struct State {
    items: Value<Vec<String>>,
}

impl State {
    pub fn new() -> Self {
        State {
            items: Value::new(vec![
                "Item 1".to_string(),
                "Item 2".to_string(),
            ]),
        }
    }

    pub fn render(&self) -> DomElement {
        dom! {
            <div>
                <p>"My list"</p>
                <ol>
                    <li>"Item 1"</li>
                    <li>"Item 2"</li>
                </ol>
            </div>
        }
    }

}
```

As you can see the method now takes self as a state definition, but do not use it yet.
Meanwhile add this sub-state into our main state in `src/state.rs`:

```rust
use crate::list;

#[derive(Clone)]
pub struct State {
    pub message: Value<String>,
    pub strong: Value<bool>,
    pub list: list::State,
}

impl State {
    pub fn component() -> DomElement {
        let state = State {
            message: Value::new("Hello world".to_string()),
            strong: Value::new(true),
            list: list::State::new(),
        };

        render(&state)
    }
}
```

Now we can use our sub-state to render our component dynamically. Back in `src/list.rs` modify `render` method this way:

```rust
    pub fn render(&self) -> DomElement {
        let elements = self.items.render_list(
            |item| item.clone(),
            |item| dom! { <li>{item}</li> },
        );

        dom! {
            <div>
                <p>"My list"</p>
                <ol>
                    { elements }
                </ol>
            </div>
        }
    }
```

The render function uses `render_list()` method on `Value<Vec<_>>` from state to render a list of `<li>` elements. The list can then be inserted directly as a list of children in `dom!` macro.
Note the `render_list()` method works only if inner type of `Value` implements `IntoIterator`.
The method takes two closures as parameters. First should return a key unique across all items, while the latter should return with the rendered item itself.

At last update main render function to use render method from our sub-state (importing list module is now not required).

```rust
    let list_state = &state.list;

    dom! {
        <div css={main_div()}>
            "Message to the world: "
            {message_element}
            <button on_click={switch}>"Switch"</button>
            {state.list.render()}
        </div>
    }
```

## 10. Input element

Our component cries out for adding more items. To implement this we need to:

1. add input element and button next to it,
2. make value of this input be taken from the `Value` stored in the state,
3. make typing in this input `change` the value in the state,
4. upon clicking on the button a closure should be fired which will `add` the value as a new element in the list and erase input value.

So the whole `src/list.rs` will look like this:

```rust
use vertigo::{Value, DomElement, dom, bind, bind2};

#[derive(Clone)]
pub struct State {
    items: Value<Vec<String>>,
    new_item: Value<String>,
}

impl State {
    pub fn new() -> Self {
        State {
            items: Value::new(vec![
                "Item 1".to_string(),
                "Item 2".to_string(),
            ]),
            new_item: Value::new(String::default()),
        }
    }

    pub fn render(&self) -> DomElement {
        let add = bind2(&self.items, &self.new_item).call(|ctx, items, new_item| {
            let mut items_vec = items.get(ctx).to_vec();
            items_vec.push(new_item.get(ctx));
            items.set(items_vec);
            new_item.set("".to_string());
        });

        let change = bind(&self.new_item).call_param(|_ctx, new_item, new_value| {
            new_item.set(new_value);
        });

        let elements = self.items.render_list(
            |item| item.clone(),
            |item| dom! { <li>{item}</li> },
        );

        dom! {
            <div>
                <p>"My list"</p>
                <ol>
                    { elements }
                </ol>
                <input value={self.new_item.to_computed()} on_input={change} />
                <button on_click={add}>"Add"</button>
            </div>
        }
    }
}
```

We've added 2 event handlers in our render function.

To create **add** handler `bind2` helper is used.
This is similar as `bind` but allows to use 2 parameters during the call. There are also helpers for 3 and 4 parameters.

For input **change** event, the `call_param` method is used to create a handler that takes value from the DOM during the call (`new_value` parameter). The type of the value is specialized after applying it in `dom!` macro.

## 11. Computed value

It is possible to have a value that is automatically computed. Let's show the amount of items in the list. First import `Computed` from `vertigo` and add a computed type to the list's state:

```rust
#[derive(Clone)]
pub struct State {
    items: Value<Vec<String>>,
    new_item: Value<String>,
    count: Computed<usize>,
}
```

Then we need to reorganize a little how we create an instance of the state:

```rust
    pub fn new() -> Self {
        let items = Value::new(vec![
            "Item 1".to_string(),
            "Item 2".to_string(),
        ]);

        let count = {
            let items = items.clone();
            Computed::from(move |ctx| items.get(ctx).len())
        };

        State {
            items,
            new_item: Value::new(String::default()),
            count,
        }
    }
```

First we need to create the list of items, then we will create the `Computed` using its `from` method which accepts a function that calculates the value. We need to clone "the access"[^clone] to the list first to be able to move it into the closure. As it was stated earlier, firing `.get(ctx)` creates a dependency in the driver's graph, so every client reading this computed will get a new value from every time the list has changed.

Now we can use this computed in render function:

```rust
    dom! {
        <div>
            <p>"My list (" { &state.count } ")"</p>
            (...)
```

## 12. Parametrized styles

As a bonus feature, we'll delve in the styles. First we'll make the list to change font color for every other row. Remember to import `css_fn` from vertigo.

```rust
css_fn! { alternate_rows, "
    color: black;

    :nth-child(odd) {
        color: blue;
    };
" }
```

And use these styles in `dom!` macro:

```rust
        let elements = self.items.render_list(
            |item| item.clone(),
            |item| dom! {
                <li css={alternate_rows()}>{item}</li>
            },
        );
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
        let elements = self.items.render_list(
            |item| item.clone(),
            |item| {
                let excl = item.ends_with('!');
                dom! {
                    <li css={alternate_rows(excl)}>{item}</li>
                }
            },
        );
```

## Further reading

Complete code for this tutorial should be found [here](https://github.com/vertigo-web/vertigo-app-template/tree/tutorial).

For any more complex scenarios please refer to the examples in the [demo](/demo/src/app) package.

[^traitaliases]: https://github.com/rust-lang/rust/issues/41517

[^watch]: You still need to refresh the page in the browser after making changes and after project rebuilds.

[^bind]: If you want to enter your app from outside your local machine then in `Makefile.toml` in section `[tasks.serve]` change `127.0.0.1` to `0.0.0.0`.

[^simplification]: This is a shameful simplification but enough for a tutorial - the correct description will be able to be found in future, more robust documentation.

[^styles]: Styles are being attached to document's `HEAD` as classes with unique auto-generated names. These names are then used in HTML tags. This way you can use such CSS functions multiple times to different HTML tags and they'll all use the same class.

[^subscription]: `get()` method creates a subscription in dependency graph so the render function is now dependent on the value, and will be fired everytime the value changes. This is similar to how the MobX library works in React world.

[^clone]: Every `Value` and `Computed` wraps it's inner value in an `Rc` so cloning does not clone the content. It just creates another pointer - a handler to access the value.
