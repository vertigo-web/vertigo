# Vertigo Tutorial
<!-- markdownlint-disable no-inline-html -->

<!-- markdownlint-disable-next-line no-emphasis-as-heading -->
*Up to date with version 0.4.2*

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
11. [Styles](#11-Styles)
12. [Parametrized styles](#12-Parametrized-styles)

## 1. Prepare your system

### rust nightly

Vertigo uses trait aliases[^traitaliases] so until it stabilizes we need rust nightly to use vertigo. The easiest way to install rust nightly is to use [rustup](https://rustup.rs/). To install nightly toolchain and switch to it, run:

- `rustup default nightly`

### wasm-opt

In order to produce smaller builds vertigo uses `wasm-opt` tool during every build.

To have `wasm-top` in your system, install `binaryen` package in your linux distribution
(f. ex. `apt-get install binaryen` on Debian/Ubuntu) or go to
[https://github.com/WebAssembly/binaryen/discussions/3797](https://github.com/WebAssembly/binaryen/discussions/3797)
for other instructions.

### vertigo-cli

Finally, let's install a command line tool which allows to create, build, serve and watch vertigo projects.

- `cargo install vertigo-cli`

## 2. Generate project

Generate project in subdirectory using command:

- `vertigo new my-vertigo-app`

## 3. First run

Enter the subdirectory:

- `cd my-vertigo-app`

The most common thing you'll be doing is watching the project. This means build it, serve locally and after every change rebuild:

- `vertigo watch`

Vertigo uses a built-in web server which supports server-side rendering.
This makes simple pages work even if the browser have JavScript turned off.

After seeing message `Listening on 127.0.0.1:4444` you can point your browser to `http://127.0.0.1:4444/` to see the "Hello World" message. The page should update automatically when the code gets edited.

## 4. Initial code description

Open `/src/lib.rs` file.

```rust
use vertigo::{main, DomNode, dom, Value};

#[main]
fn app() -> DomNode {
    let message = Value::new("world");
    dom! {
        <html>
            <head />
            <body>
                <div>"Hello " {message}</div>
            </body>
        </html>
    }
}
```

This is the main entry point for the application. It creates a very simple state (a string message) and transforms this state into a `DomElement`.

Let's outline a little bigger picture now.

Vertigo app mainly consists of four parts[^simplification]:

- *Dependency graph* - which holds the current state of app and triggers its leaf clients upon some change,
- *DOM elements* - that can be deps graph's clients and know how to update itself on the page,
- *HTML/CSS macros* - which provides a convenient way to create DOM elements.
- *Server-side rendering* - Out of the box when using `vertigo-cli`

If we want to be a little more detailed in this description, then it would be:

- Dependency graph holds values, computed values (computeds) and clients (mount functions).
- Upon changing some value all dependent computeds get computed, and all dependent clients get updated.
- Mount function takes a computed state provided by the graph and returns a "render definition" (`DomElement`).
It is important to remember that `DomElement` is not a product of render process. It is a **definition of a render process**.
- Upon any change in state, DOM is also updated if necessary.
- Mount functions can provide the DOM with functions that get fired on events like `on_click`,
which may modify the state, thus triggering necessary computing once again.
- Coupled state and mount function is called component.
- Components (connected with themselves in a parent-child hierarchy) make a reactive website.
- HTML can be prepared server-side when using `vertigo serve` (or `watch`) command,
this speeds-up page loads and allows web crawlers to properly index your website.

Now let's breakdown the code line by line:

```rust
use vertigo::{main, DomNode, dom, Value};
```

Here we import:

- `main` - Marks an entry point of the app
- `DomElement` - a struct that will define output of our mount function (a reactive component),
- `dom!` - a macro to use HTML tags to define the shape of the resultant element, and
- `Value` - a reactive box for values,

```rust
#[main]
fn app() -> DomNode {
```

This is our main "render" function, but in fact it is a mount function (fired only once because updates are
performed through dependency graph). It returns a definition of a reactive DOM in the form of `DomElement` instance.

It is decorated with `main` macro which wraps the function in a gateway from JavaScript world.

```rust
    let message = Value::new("world!".to_string());
```

Here we create a very simple state which only hold a String value. This is performed inside the mount function,
which means the state will be created from scratch every time our component gets mounted (but not on every render!).

```rust
    dom! {
```

The `dom!` macro always returns `DomElement` object so it usually is at the end of the function which returns the same type.
You may as well pre-generate parts of the component using this macro and use it in the body of another `dom!` invocation.

```rust
        <html>
            <head />
            <body>
```

Because this is a root component, we need to define all the required HTML elements needed to form a website.
Keep in mind that the `<head />` element is necessary for vertigo, so even if you don't want to use it
you need at least place such an empty tag in your HTML.

```rust
                <div>"Hello " {message}</div>
```

Next, in the `div` we insert a text node. Strings in `dom!` macro must always be double-quoted. This assures us we won't miss a space between the text and the next DOM element.

Next we're inserting a value from the state. The state is of type `Value<String>` which can be directly embedded into
`dom!` macro without any transformations.

## 5. Add new value

For starters let's add some conditional rendering, depending on some state.

This is our modified `app` function:

```rust
#[main]
fn app() -> DomNode {
    let message = Value::new("world!");
    let strong = Value::new(true);

    let message_element = strong.render_value(move |strong|
        if strong {
            dom! { <strong>{&message}</strong> }
        } else {
            dom! { <span>{&message}</span> }
        }
    );

    dom! {
        <html>
            <head />
            <body>
                <div>"Hello " {message_element}</div>
            </body>
        </html>
    }
}
```

In the browser the message should be now in bold.

Few words of explanation. Conditional rendering is performed here by a `Value::render_value` function which allows to
create something like an anonymous component, where state is the `strong` value, and mount function is a closure passed
as an argument. The closure needs `message` value from outside, hence the `move` keyword.
If the `message` cannot be moved because it is used later, then it should be cloned first.
`Value` implements shallow cloning[^clone], so it's perfectly ok to clone it whenever it is needed.

`message_element` is of type `DomElement` so it can be used in `dom!` macro directly in the same way as string.

## 6. Set value

Let's do some reactivity already. Import `vertigo::bind` and add switch event just before the `dom!` macro:

```rust
    let switch = bind!(strong, ||
        strong.change(|val| { *val = !*val; })
    );

    dom! {
        <html>
            <head />
            <body>
                <div>"Hello " {message_element}</div>
                <button on_click={switch}>"Switch"</button>
            </body>
        </html>
    }
```

To create an event handler in a handy way, vertigo introduces a "bind!" macro.
This reminds a `.bind()` function from JavaScript world, but the reason is different.
Binding a value automatically creates a clone of the value that can be used upon firing the event.

There is also a `<button>` added in the `dom!` macro firing the event upon clicking.

## 7. New component

No app should be written as one big function. Here is how we can add a component to our app. Create file `src/list.rs`:

```rust
use vertigo::{component, dom};

#[component]
pub fn List() {
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
use list::List;
```

And use it in main `app()` function:

```rust
    dom! {
        <html>
            <head />
            <body>
                <div>"Hello " {message_element}</div>
                <button on_click={switch}>"Switch"</button>
                <List />
            </body>
        </html>
    }
```

Why the function name starts from uppercase letter?
The `#[component]` macro transforms the function into a struct named `List`, with `mount` method.
This will be handy later for defining component properties,
but for now we use it just to have the component name look good.

## 8. Add state to component

For now our component just shows a static list which is not the usual way of rendering lists.
To go dynamic, add state `elements` to the `List` function and use it during rendering:

```rust
use vertigo::{component, dom, Value};

#[component]
pub fn List() {
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
                {elements}
            </ol>
        </div>
    }
}
```

We can create state this way, because this function is fired upon mounting, not every time something changes.

> The render function uses `render_list()` method on `Value<Vec<_>>` from state to render a list of `<li>` elements.
>
> The list can then be inserted directly as a list of children in `dom!` macro.
> The method takes two closures as parameters. First should return a key unique across all items,
> while the latter should return with the rendered item itself.
>
> Note the `render_list()` method works only if inner type of `Value` implements `IntoIterator`.

If we want to provide the state for a component from the upstream, then we can take the state as an argument
to the `List` function, then the state can be passed to our component as a property:

```rust
#[component]
pub fn List(items: Value<Vec<String>>) {
    let elements = items.render_list(
        |item| item.clone(),
        |item| dom! { <li>{item}</li> },
    );

    dom! {
        <div>
            <p>"My list"</p>
            <ol>
                {elements}
            </ol>
        </div>
    }
}
```

Set of arguments taken by such function is similar to `props` in React.
To make it work we need to move creation of the state to our main app and provide it to the `List` component:

```rust
fn app() -> DomNode {
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
        <html>
            <head />
            <body>
                <div>"Hello " {message_element}</div>
                <button on_click={switch}>"Switch"</button>
                <List items={my_items} />
            </body>
        </html>
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
use vertigo::{bind, component, dom, transaction, Value};

#[component]
pub fn List(items: Value<Vec<String>>) {
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

    let elements = items.render_list(
        |item| item.clone(),
        |item| dom! { <li>{item}</li> },
    );

    dom! {
        <div>
            <p>"My list"</p>
            <ol>
                {elements}
            </ol>
            <input value={new_item.to_computed()} on_input={change} />
            <button on_click={add}>"Add"</button>
        </div>
    }
}

```

We've added 2 event handlers in our mount function.

To create **add** handler, a `transaction` function is used. It allows to do more modifications in one run
(without re-rendering), and also allows to use more values at the same time.

> Keep in mind though, that values are kept the same during transaction, and only changed during next graph recalculation.
>
> Transaction provides `Context` which allows you to unwrap `Value` for the time of transaction and use it as a regular variable (`get(ctx)`[^subscription] method on a `Value`).

For **change** event, we are getting `new_value` as an argument to the closure. This is a value passed from DOM when executing event handler. The type of the value is specialized after applying it in `dom!` macro.

## 10. Computed value

It is possible to have a value that is automatically computed. Let's show the amount of items in the list. First import `Computed` from `vertigo` and add to the `List` function just after creating `elements`:

```rust
    let count = items.map(|items| items.len());
```

And use it in `dom!` macro:

```rust
    dom! {
        <div>
            <p>"My list (" {count} ")"</p>
        // (...)
```

Map on a value creates a reactive computed value. It gets updated every time the original value is changed.

## 11. Styles

Let's paint our site with a little colors. To style a div import `css`[^styles] from vertigo to `lib.rs` and add

```rust
let title_style = css!("
    color: darkblue;
");
```

before `dom!` macro usage. In the macro, change the line with div to:

```rust
<div css={title_style}>"Hello " {message_element}</div>
```

The title text should now be dark blue.

Styles can be nested in similar way to SCSS, just without the & sign.
We'll make the list change font color for every other row (`list.rs` file).

```rust
    let alternate_rows = css!("
        color: black;

        :nth-child(odd) {
            color: blue;
        };
    ");

    let elements = items.render_list(
        |item| item.clone(),
        move |item| dom! {
            <li css={alternate_rows.clone()}>{item}</li>
        },
    );
```

</details>

## 12. Parametrized styles

Say we want to have particular items emphasized by different background.
And all items ending with an exclamation mark.
To create a parameterized css we just need turn our css into a function returning styles:

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

And here's the usage in rendering:

```rust
    let elements = items.render_list(
        |item| item.clone(),
        move |item| {
            let excl = item.ends_with('!');
            dom! {
                <li css={alternate_rows(excl)}>{item}</li>
            }
        },
    );
```

## Further reading

Complete code for this tutorial should be found [here](https://github.com/vertigo-web/vertigo-tutorial/tree/master).

For any more complex scenarios please refer to [examples](/examples) and [demo](/demo/src/app) package.

[^traitaliases]: https://github.com/rust-lang/rust/issues/41517

[^simplification]: This is a shameful simplification but enough for a tutorial - the correct description will be able to be found in future, more robust documentation.

[^clone]: Every `Value` and `Computed` wraps it's inner value in an `Rc` so cloning does not clone the content. It just creates another pointer - a handler to access the value.

[^subscription]: `get()` method creates a subscription in dependency graph so the render function is now dependent on the value, and will be fired every time the value changes. This is similar to how the MobX library works in React world.

[^styles]: Styles generated by this macro are being attached to document's `HEAD` as classes with unique auto-generated names. These names are then used in HTML tags. This way you can use such CSS functions multiple times to different HTML tags and they'll all use the same class.
