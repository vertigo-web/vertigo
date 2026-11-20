# Writing apps with vertigo — rules for coding agents

This page is for an AI coding agent, or a human in a hurry, writing an application **with**
vertigo. It lists the idioms to use and the habits from React, Leptos, Yew or Dioxus that don't
carry over. Every snippet here is a doctest, so it compiles against the version of vertigo it
ships with.

## Before writing any code

**Don't trust what you remember about vertigo.** The API has changed a lot between 0.x
releases and your training data mixes versions. Read the source of the version the project
actually uses. Find it with:

```sh
cargo metadata --format-version 1 \
  | jq -r '.packages[] | select(.name == "vertigo") | .manifest_path'
```

This works for crates.io, git and path dependencies. Without `jq`, look in
`~/.cargo/registry/src/*/vertigo-<version>/`, taking the version from `Cargo.lock`. That
directory has:

- `docs/` - this file and the long-form guides (reactive graph, list caches, websockets)
- `src/lib.rs` - the crate root, with a doc example for every macro
- `CHANGELOG.md` - what changed, version by version (read it when upgrading)

Grep the source for a type or method before using it. The examples and the demo app on GitHub
(`examples/` and `demo/app/src/` in the vertigo repository) show nearly every feature in use.

## Mental model

- **A component function runs once, when it mounts, not on every change.** There is no
  re-render of the function, no hooks, no `use_state`. Create state at the top of the function
  and it lives as long as the component does.
- State is `Value<T>` (read-write) and `Computed<T>` (derived, read-only). Both are cheap
  `Rc` handles: cloning one gives another handle to the same value, so clone freely.
- `dom!` builds real DOM nodes bound to those values. When a value changes, only the nodes
  that read it update. There is no virtual DOM and no diffing.
- Everything is single-threaded. Use `Rc`, `RefCell` and `Value`, never `Arc` or `Mutex`.
- The same wasm runs on the server to render the page (SSR) and then in the browser, which
  takes over the server's HTML (hydration).

## The app and its components

The entry point is a `#[main]` function returning the whole document. `<html>`, `<head/>` and
`<body>` are required, even when `<head/>` is empty. Keep `#[main]` at the end of `lib.rs`:
tailwind classes used after it, or in its body, are left out of the bundle.

A component is a function marked `#[component]`, named in `PascalCase`, with **no return
type**. Its body ends with `dom!`. Its parameters are its props:

```rust
use vertigo::{Value, bind, component, dom};

#[component]
pub fn Counter(label: String, count: Value<i32>) {
    let increment = bind!(count, |_| count.change(|c| *c += 1));

    dom! {
        <div>
            <span>{label} ": " {count}</span>
            <button on_click={increment}>"+"</button>
        </div>
    }
}

let count = Value::new(0);

dom! {
    <Counter label="Clicks" {count} />
};
```

`{count}` in a tag is short for `count={count}`. For optional props see `#[default]` in the
docs of [`component`](crate::component). To pass arbitrary attributes down to a child element,
take an [`AttrGroup`](crate::AttrGroup) prop.

## `dom!` is not JSX

- **Text must be a quoted string literal**: `<p>"Hello " {name}</p>`, never `<p>Hello</p>`.
- `{expr}` embeds anything `Display`, a `Value<T>` or `Computed<T>` (it updates by itself), a
  `DomNode`, or a reference to any of these.
- **No `if`, `match` or `for` inside `dom!`.** Compute the node before the macro and embed it:
  - an `Option<DomNode>` with the spread operator: `{..maybe_node}`,
  - a node that depends on reactive state with `render_value` (below),
  - a list with `render_list` (below).
- Events are attributes named `on_*`. The common ones and what their handler gets:
  - `on_click` - a [`ClickEvent`](crate::ClickEvent) (usually ignored: `|_|`), with the modifier keys
    and where the pointer clicked, f. ex. on a `<canvas>`
  - `on_input`, `on_change` - the new `String` value
  - `on_change_checked` - the new `bool`
  - `on_key_down` - a [`KeyDownEvent`](crate::KeyDownEvent). Return `true` when you handled the key, to stop
    its default action.

  Find all of them as the `on_*` methods of [`DomElement`](crate::DomElement).
- Styles: `css={...}` with a [`css!`](crate::css) value, or tailwind with `tw="flex gap-2"`.
  Plain `class="..."` works too.

```rust
use vertigo::{css, dom};

let name = "Ada";
let nickname: Option<&str> = None;

let badge = css! {"
    font-weight: bold;

    :hover {
        color: red;
    }
"};

dom! {
    <div tw="flex gap-2">
        <span css={badge}>{name}</span>
        {..nickname.map(|nick| dom! { <span>"aka " {nick}</span> })}
    </div>
};
```

A `css!` block can embed Rust values: `color: {my_color};`. A function returning
[`Css`](crate::Css), or a `Computed<Css>` (`value.map(|v| ...css!...)`), gives styles that
depend on state.

## Reading and writing state

There is no `value.get()` without a context. Where you read decides how:

- **To show a value**, embed it in `dom!` directly. Don't read it at all.
- **To derive a value**, use `.map(...)` on a `Value` or `Computed`, or `Computed::from(|ctx| ...)`
  to combine several. These re-run when what they read changes.
- **To read in an event handler or a spawned task**, open a `transaction(|ctx| ...)`.

```rust
use vertigo::{Computed, Value, transaction};

let first = Value::new("Ada".to_string());
let last = Value::new("Lovelace".to_string());

// Derived: recomputed whenever `first` or `last` changes.
let full = Computed::from({
    let (first, last) = (first.clone(), last.clone());
    move |ctx| format!("{} {}", first.get(ctx), last.get(ctx))
});
let length = full.map(|name| name.len());

// A write. `change` modifies in place, `set` replaces.
first.set("Augusta Ada".to_string());

// A plain read, in a transaction.
transaction(|ctx| {
    assert_eq!(full.get(ctx), "Augusta Ada Lovelace");
    assert_eq!(length.get(ctx), 20);
});
```

Rules that are easy to break:

- **Never write in a computation.** A `set` or `change` inside `.map`, `Computed::from` or
  a render closure is ignored and reported. Write in event handlers, spawned tasks or
  transactions.
- **Inside a transaction, only the `Value` you wrote changes right away.** Every `Computed`
  derived from it, and the DOM, catch up when the transaction ends. Don't write a `Value` and
  then read a `Computed` built on it in the same transaction.
- `set` with a value equal to the current one does nothing, which is why `T` needs
  `PartialEq`. `get` clones `T`, so wrap large data in `Rc`.

## Event handlers: `bind!`

[`bind!`](crate::bind) clones the listed handles into the closure, so you don't write
`let x = x.clone(); move |_| ...` by hand. For async handlers use
[`bind_spawn!`](crate::bind_spawn), which runs the returned future.

```rust
use vertigo::{Value, bind, bind_spawn, dom, get_driver, transaction};

let text = Value::new(String::new());
let items = Value::new(Vec::<String>::new());

let on_input = bind!(text, |new_text: String| text.set(new_text));

let add = bind!(text, items, |_| {
    transaction(|ctx| {
        let new_item = text.get(ctx);
        items.change(|items| items.push(new_item));
        text.set(String::new());
    });
});

let add_later = bind_spawn!(items, |_| async move {
    get_driver().sleep(500).await;
    items.change(|items| items.push("late".to_string()));
});

dom! {
    <div>
        <input value={text.to_computed()} {on_input} />
        <button on_click={add}>"Add"</button>
        <button on_click={add_later}>"Add in a moment"</button>
    </div>
};
```

## Conditional content and lists

```rust
use vertigo::{Value, dom, render::render_list};

let logged_in = Value::new(false);
let todos = Value::new(vec!["Write docs".to_string(), "Ship".to_string()]);

// Re-renders this part whenever `logged_in` changes.
let greeting = logged_in.render_value(|logged_in| {
    if logged_in {
        dom! { <p>"Welcome back"</p> }
    } else {
        dom! { <p>"Please log in"</p> }
    }
});

// Keyed list: a row is only rebuilt when its key appears.
let list = render_list(
    &todos,
    |todo| todo.clone(),            // key: unique, `Clone + Eq + Hash + Debug`
    |todo| dom! { <li>{todo}</li> }, // `todo` is a `&Computed<String>` for this row
);

dom! {
    <div>
        {greeting}
        <ul>{list}</ul>
    </div>
};
```

`render_value_option` is the same, for a closure returning `Option<DomNode>`. For lists that
are edited (CRUD with optimistic updates) read the [`LazyListCache`](crate::guides::lazy_list_cache)
guide instead of rolling your own.

## Global state: `#[store]`

A function marked [`#[store]`](crate::store) runs once and every call returns a handle to the
same value: an app-wide singleton. With arguments, it runs once per distinct argument. Use it
instead of `static`, `lazy_static`, `OnceCell` or `Arc<Mutex<_>>`.

```rust
use vertigo::{Value, store};

#[store]
fn state_dark_mode() -> Value<bool> {
    Value::new(false)
}

state_dark_mode().set(true);
```

## Talking to a server

Not `reqwest`, `gloo`, `web-sys`, `tokio` or `serde_json`. Vertigo has its own fetch, JSON and
task runner, and they also work during server-side rendering.

- **Data to show**: a [`LazyCache`](crate::LazyCache) made by
  [`RequestBuilder`](crate::RequestBuilder), kept in a `#[store]`. It fetches when something
  first reads it, keeps the value for its TTL, and has `Loading`, `Ready` and `Error` states.
- **Actions** (POST, PUT, DELETE): `RequestBuilder::post(..).body_json(..).call().await` in a
  spawned task, then update state, or `.forget()` the caches that showed the old data.
- **JSON types**: `#[derive(AutoJsJson)]`, not serde.

```rust
use vertigo::{AutoJsJson, LazyCache, RequestBuilder, bind_spawn, dom, store};

#[derive(AutoJsJson, Clone, PartialEq)]
struct Post {
    id: u64,
    title: String,
}

#[derive(AutoJsJson)]
struct NewPost {
    title: String,
}

#[store]
fn posts() -> LazyCache<Vec<Post>> {
    RequestBuilder::get("/api/posts")
        .ttl_minutes(5)
        .lazy_cache(|status, body| (status == 200).then(|| body.into()))
}

// `render` shows a default loading and error state; to show your own, match on
// `posts().to_computed()`, a `Computed<Resource<Rc<Vec<Post>>>>`.
let list = posts().render(|posts| {
    dom! {
        <ul>
            {..posts.iter().map(|post| dom! { <li>{&post.title}</li> })}
        </ul>
    }
});

let add = bind_spawn!(|_| async move {
    let response = RequestBuilder::post("/api/posts")
        .body_json(NewPost { title: "Hello".to_string() })
        .call()
        .await;
    match response.into(|status, body| (status == 201).then(|| body.into::<Post>())) {
        Ok(_) => posts().forget(), // fetched again by whatever shows it
        Err(err) => vertigo::log::error!("Can't add post: {err}"),
    }
});

dom! {
    <div>
        {list}
        <button on_click={add}>"Add post"</button>
    </div>
};
```

Relative URLs like `/api/posts` work in the browser. During SSR the server needs to know where
to send them: the fullstack template sets `ssr_fetch_base` for this. Data that depends on
cookies needs care during SSR, see the [`LazyCache`](crate::LazyCache) docs.

## Routing

[`Router`](crate::router::Router) keeps the address in a value of your route enum, which needs
`Clone + PartialEq + Display + From<String>`. Render with `router.route.render_value(...)`.
Navigate with a plain `<a href=...>` (a history router handles it without a page load) or with
`router.set(route)`. See `examples/router` in the vertigo repository.

## The browser, through vertigo

The target is `wasm32-unknown-unknown` without wasm-bindgen. **Crates that need wasm-bindgen,
`web-sys` or `js-sys` won't work**: not `gloo`, not `reqwest`, not `getrandom` with its `js`
(0.2) or `wasm_js` (0.3 and later) feature, and not crates depending on those. Many `std` functions panic or do nothing there.

| Instead of | Use |
| --- | --- |
| `println!`, `eprintln!`, `dbg!` | `log::info!` and friends (re-exported as `vertigo::log`), shown in the browser console |
| `std::time::Instant`, `SystemTime` | `get_driver().now()`, `get_driver().utc_now()` |
| `thread::sleep`, `tokio::time` | `get_driver().sleep(ms).await`, `get_driver().set_interval(..)` |
| `thread::spawn`, `tokio::spawn` | `vertigo::spawn(future)`, `bind_spawn!` |
| `rand` | `get_driver().get_random(min, max)` |
| `std::env::var`, config files | `get_driver().env("NAME")`, set with `vertigo watch --env NAME=value` |
| `document.cookie`, `localStorage`-style | `get_driver().cookie_get(..)`, `cookie_set(..)` |
| `web_sys` canvas contexts, `glow`, `wgpu` | a [`Drawing`](crate::Drawing) in `<canvas draw={..}>` for 2D, [`webgl::WebGl`](crate::webgl::WebGl) for WebGL 2 |
| `web_sys` for anything else | a [`Driver`](crate::Driver) method first, then [`js!`](crate::js) |
| a file in `static/` | [`include_static!`](crate::include_static) gives its public path |

## Server-side rendering

Your component code runs on the server first, then again in the browser.

- Keep the first render the same on both sides. Random numbers or the current time in it make
  the server's HTML differ from what the browser renders.
- Guard browser-only work (`js!`, focusing an element, measuring layout) with
  `get_driver().is_browser()`.

## Building and checking

- `vertigo watch` builds, serves on <http://127.0.0.1:4444> and rebuilds on every change. It
  never exits: run it in the background and don't wait for it.
- `vertigo build` does one build. It is the quickest proof that the app compiles for the
  browser. `cargo clippy --target wasm32-unknown-unknown` lints the same code.
- Fullstack projects: the `justfile` has `watch-fe` for the frontend and `run-be` for the
  backend, which builds the frontend first.

## Further reading

- [`guides::reactive_graph`](crate::guides::reactive_graph) - how `Value`, `Computed` and
  subscriptions fit together
- [`guides::collection_key_and_list_renderers`](crate::guides::collection_key_and_list_renderers)
  - `CollectionKey` and the memoized list renderers
- [`guides::lazy_list_cache`](crate::guides::lazy_list_cache) - optimistic CRUD lists
- [`guides::websocket_collection`](crate::guides::websocket_collection) - collections pushed by
  the server over a WebSocket
