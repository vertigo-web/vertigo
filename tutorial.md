# Vertigo Tutorial

<!-- markdownlint-disable-next-line no-emphasis-as-heading -->
*Up to date with version 0.2.0-alpha*

<!-- markdownlint-disable-next-line heading-increment -->
### Table of contents

<!-- markdownlint-disable link-fragments -->

1. [Prepare your system](#1-Prepare-your-system)
2. [Generate project](#2-Generate-project)
3. [First run](#3-First-run)
4. [Initial code description](#4-Initial-code-description)
5. [Add new value](#5-Add-new-value)
6. [Set value](#6-Set-value)
7. [New component](#7-New-component)
8. [Add state to component](#8-Add-state-to-component)
9. [Input element](#9-Input-element)
10. [Computed value](#10-Computed-value)
11. [Parametrized styles](#11-Parametrized-styles)

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

## 4. Initial code description

Open `/src/render.rs` file.

```rust
use vertigo::{start_app, DomElement, Value, dom, css};

fn app(message: &Value) -> DomElement {
    let main_div = css!("
        color: darkblue;
    ");

    dom! {
            <div css={main_div}>
            "Message to the world: "
            { message }
        </div>
    }
}

#[no_mangle]
pub fn start_application() {
    let message = Value::new("Hello world!".to_string());
    let view = app(&message);
    start_app(message, view);
}
```

This is the main entry point for the application. It creates a vary simple state (a string message) and transforms it into a `DomElement`.

Vertigo app mainly consists of three parts[^simplification]:

- *Dependency graph* - which holds the current state of app and triggers its leaf clients upon some change,
- *DOM elements* - that can be deps graph's clients and know how to update itself on the page,
- *HTML/CSS macros* - which provides a convenient way to create VDOM elements.

If we want to be a little more detailed in this description, then it would be:

- Dependency graph holds values, computed values (computeds) and clients (mount/render functions).
- Upon changing some value all dependent computeds get computed, and all dependent clients get updated.
- Mount (or render) function takes a computed state provided by the graph and returns a rendered element (`DomElement`).
- Upon any change in state, DOM is also updated if necessary.
- Mount functions can provide the DOM with functions that get fired on events like `on_click`, which may modify the state, thus triggering necessary computing once again.
- Coupled state and mount function is called component.

Now let's breakdown the code line by line:

```rust
use vertigo::{start_app, DomElement, Value, dom, css};
```

Here we import `start_app` function which initializes vertigo env, and creates a root node.

We also import:

- `DomElement` - a struct that will define output of our mount function (a reactive component),
- `Value` - a reactive box for values,
- `dom!` - a macro to use HTML tags to define the shape of the resultant element, and
- `css!` macro that defines styles for DOM nodes using CSS syntax.

```rust
fn app() -> DomElement {
```

This is our main "render" function, but in fact this is a "mount" function (fired only once because updates are
performed through dependency graph). Tt just return definition of a reactive DOM in the form of `DomElement` instance.

```rust
    let message = Value::new("Hello world!".to_string());
```

Here we create a very simple state which only hold a String value. This is performed inside the mount function,
which means the state will be created from scratch every time our component gets mounted.

```rust
let main_div = css!("
    color: darkblue;
");
```

Using `css!` macro we define here styles[^styles] with `color: darkblue` body ready to use in `dom!` macro.

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
            { message }
```

Here we're inserting some value from the state. The `message` field in the state is of type `Value<String>`
which can be directly embedded into `dom!` macro without any transformations.

```rust
        </div>
```

The `div` tag must be of course closed as in regular HTML.

```rust
#[no_mangle]
pub fn start_application() {
    start_app(message, view);
}
```

Then the root function of our app is passed to `start_app` function which is fired during `start_application`
which acts as a hardcoded gateway from JavaScript world (hence the `#[no_mangle]` attribute)

## 5. Add new value

For starters let's add a new boolean value to the state and use it to render the component conditionally.
Add a boolean value next to the message value, and use it to conditionally render message in different way:

```rust
    let message = Value::new("Hello world!".to_string());
    let strong = Value::new(true);
```

to `State::component()` method. Then in render function you can use this value:

```rust
pub fn render(state: &State) -> DomElement {
    let message = Value::new("Hello world!".to_string());
    let strong = Value::new(true);

    let message_element = strong.render_value(move |strong|
        if strong {
            dom! { <strong>{&message}</strong> }
        } else {
            dom! { <span>{&message}</span> }
        }
    );

    dom! {
        <div css={main_div()}>
            "Message to the world: "
            { message_element }
        </div>
    }
}
```

In the browser the message should be now in bold.

## 6. Set value

Let's do some reactivity already. Import `vertigo::bind`, add switch event and use it in `dom!` macro:

```rust
    let switch = bind!(strong, ||
        strong.change(|val| { *val = !*val; })
    );

    dom! {
        <div css={main_div}>
            "Message to the world: "
            {message_element}
            <button on_click={switch}>"Switch"</button>
        </div>
    }
```

To create an event handler in a handy way, vertigo introduces a "bind!" macro.
This reminds a `.bind()` function from JavaScript world, but the reason is different.
Binding a value automatically creates a clone of the value that can be used upon firing the event.
Happily enough, everything wrapped in a `Value<T>` have a shallow cloning implemented[^clone].

## 7. New component

No app should be written as one big render function. Here how we can add a component to our app. Create file `src/list.rs`:

```rust
use vertigo::{DomElement, dom};

pub struct List { }

impl List {
    pub fn mount(self) -> DomElement {
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

Add to `/src/lib.rs`:

```rust
mod list;
use list::List;
```

And use it in main `app()` function:

```rust
    dom! {
        <div css={main_div}>
            "Message to the world: "
            {message_element}
            <button on_click={switch}>"Switch"</button>
            <List />
        </div>
    }
```

## 8. Add state to component

For now our component just shows a static list which is not the usual way of rendering lists.
To go dynamic, add state `elements` to the `mount` function and use it during rendering:

```rust
use vertigo::{DomElement, dom, Value};

pub struct List { }

impl List {
    pub fn mount(self) -> DomElement {
        let items = vec![
            "Item1".to_string(),
            "Item2".to_string(),
        ];

        let state = Value::new(items);

        let elements = state.render_list(
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
}
```

We can create state this way, because this function is fired upon mounting, not everything something changes.

> The render function uses `render_list()` method on `Value<Vec<_>>` from state to render a list of `<li>` elements.
>
> The list can then be inserted directly as a list of children in `dom!` macro.
> The method takes two closures as parameters. First should return a key unique across all items,
> while the latter should return with the rendered item itself.
>
> Note the `render_list()` method works only if inner type of `Value` implements `IntoIterator`.

If we want to provide the state for a component from the upstream, then we can move the state to the struct `List` itself,
then the state can be passed to our component as a property:

```rust
use vertigo::{DomElement, dom, Value};

pub struct List {
    pub items: Value<Vec<String>>,
}

impl List {
    pub fn mount(self) -> DomElement {
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
}
```

As you can see the method takes self as a parameter. This is similar to `props` in React.
To make it work we need to move creation of the state to our main app and provide it to the `List` component:

```rust
fn app() -> DomElement {
    let message = Value::new("Hello world!".to_string());
    let strong = Value::new(true);

    let my_items = Value::new(
        vec![
            "Item1".to_string(),
            "Item2".to_string(),
        ]
    );

    // (...)
```

and add it as a property:

```rust
    dom! {
        <div css={main_div}>
            "Message to the world: "
            {message_element}
            <button on_click={switch}>"Switch"</button>
            <List items={my_items}/>
        </div>
    }
```

## 9. Input element

Our component cries out for adding more items. To implement this we need to:

1. add input element and button next to it,
2. make value of this input be taken from the `Value` stored in the state,
3. make typing in this input `change` the value in the state,
4. upon clicking on the button a closure should be fired which will `add` the value as a new element in the list and erase input value.

So the whole `src/list.rs` will look like this:

```rust
use vertigo::{DomElement, dom, Value, bind, transaction};

pub struct List {
    pub items: Value<Vec<String>>,
}

impl List {
    pub fn mount(self) -> DomElement {
        let new_item = Value::<String>::default();

        let add = bind!(items, new_item, || {
            transaction(|ctx| {
                items.change(|items| items.push(new_item.get(ctx)));
                new_item.set("".to_string());
            });
        });

        let change = bind!(new_item, |new_value| {
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
                <input value={new_item.to_computed()} on_input={change} />
                <button on_click={add}>"Add"</button>
            </div>
        }
    }
}
```

We've added 2 event handlers in our mount function.

To create **add** handler, a `transaction` function is used. It allows to do more modifications in one run, and also allows to use more values at the same time.

> Keep in mind though, that values are kept the same during transaction, and only changed during next graph recalculation.
>
> Transaction provides `Context` which allows you to unwrap `Value` for the time of transaction and use it as a regular variable (`get(ctx)`[^subscription] method on a `Value`).

For input **change** event, we are getting `new_value` in the closure. This is a value passed from DOM when executing event handler. The type of the value is specialized after applying it in `dom!` macro.

## 10. Computed value

It is possible to have a value that is automatically computed. Let's show the amount of items in the list. First import `Computed` from `vertigo` and add to the mount function just after creating items list:

```rust
        let count = self.items.map(|items| items.len());
```

And use it in `dom!` macro:

```rust
        dom! {
            <div>
                <p>"My list (" {count} ")"</p>
            // (...)
```

Map on a value creates a reactive computed value. It gets updated every time the original value is changed.

## 11. Parametrized styles

As a bonus feature, we'll delve into styles. First we'll make the list to change font color for every other row.
Remember to import `css` from vertigo. In `list.rs` file add:

```rust
        let alternate_rows = || css!("
            color: black;

            :nth-child(odd) {
                color: blue;
            };
        ");
```

We've created "css factory" (that is a closure) because we're using this style in every iteration here:

```rust
        let elements = self.items.render_list(
            |item| item.clone(),
            |item| dom! {
                <li css={alternate_rows()}>{item}</li>
            },
        );
```

Now we want to have particular items emphasized by different background. Let's say all items ending with an exclamation mark.
To create a parameterized css function we just need add parameter to our factory:

```rust
        let alternate_rows = |excl: bool| {
            let bg_color = if excl { "yellow" } else { "inherit" };

            css!("
                color: black;
                background: { bg_color };

                :nth-child(odd) {
                    color: blue;
                };
            ")
        };
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

[^clone]: Every `Value` and `Computed` wraps it's inner value in an `Rc` so cloning does not clone the content. It just creates another pointer - a handler to access the value.

[^subscription]: `get()` method creates a subscription in dependency graph so the render function is now dependent on the value, and will be fired everytime the value changes. This is similar to how the MobX library works in React world.
