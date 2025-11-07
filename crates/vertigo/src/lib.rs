//! Vertigo is a library for building reactive web components.
//!
//! It mainly consists of four parts:
//!
//! * **Reactive dependencies** - A graph of values and clients (micro-subscriptions)
//!   that can automatically compute what to refresh after one or more value change(s)
//! * **Real DOM** - No intermediate Virtual DOM mechanism is necessary
//! * **HTML/CSS macros** - Allows to construct Real DOM nodes using HTML and CSS
//! * **Server-side rendering** - Out of the box when using `vertigo-cli`
//!
//! ## Example 1
//!
//! ```rust
//! use vertigo::{dom, DomNode, Value, bind, main};
//!
//! #[main]
//! pub fn app() -> DomNode {
//!     let count = Value::new(0);
//!
//!     let increment = bind!(count, |_| {
//!         count.change(|value| {
//!             *value += 1;
//!         });
//!     });
//!
//!     let decrement = bind!(count, |_| {
//!         count.change(|value| {
//!             *value -= 1;
//!         });
//!     });
//!
//!     dom! {
//!         <html>
//!             <head/>
//!             <body>
//!                 <div>
//!                     <p>"Counter: " { count }</p>
//!                     <button on_click={decrement}>"-"</button>
//!                     <button on_click={increment}>"+"</button>
//!                 </div>
//!             </body>
//!         </html>
//!     }
//! }
//! ```
//!
//! ## Example 2
//!
//! ```rust
//! use vertigo::{css, component, DomNode, Value, dom, main};
//!
//! #[component]
//! pub fn MyMessage(message: Value<String>) {
//!     dom! {
//!         <p>
//!             "Message to the world: "
//!             { message }
//!         </p>
//!     }
//! }
//!
//! #[main]
//! fn app() -> DomNode {
//!     let message = Value::new("Hello world!".to_string());
//!
//!     let main_div = css!("
//!         color: darkblue;
//!     ");
//!
//!     dom! {
//!         <html>
//!             <head/>
//!             <body>
//!                 <div css={main_div}>
//!                     <MyMessage message={message} />
//!                 </div>
//!             </body>
//!         </html>
//!     }
//! }
//! ```
//!
//! To get started you may consider looking at the
//! [Tutorial](https://github.com/vertigo-web/vertigo/blob/master/tutorial.md).
//!
//! Short-links to most commonly used things:
//!
//! * [dom!] - Builds [DomNode] using RSX/rstml (HTML-like) syntax
//! * [css!] - Builds [Css] using CSS-like syntax
//! * [component] - Wraps function to be used as component in RSX
//! * [main] - Wraps function to be vertigo entry-point
//! * [get_driver] - Access to browser facilities
//! * [bind!] - Auto-clones variables before use
//! * [Value] - Read-write reactive value
//! * [Computed] - Read-only (computed) reactive value
//! * [router::Router] - Hash or history routing
//! * [store] - Wraps function to be used as a store generator

#![deny(rust_2018_idioms)]
#![feature(try_trait_v2)] // https://github.com/rust-lang/rust/issues/84277

mod computed;
mod css;
mod dom;
mod dom_macro;
mod driver_module;
mod fetch;
mod future_box;
pub mod inspect;
mod instant;
mod long_ptr;
mod render;
pub mod router;

#[cfg(test)]
mod tests;
mod websocket;

use std::rc::Rc;

use computed::struct_mut::ValueMut;

pub use computed::{
    context::Context, struct_mut, AutoMap, Computed, Dependencies, DropResource, GraphId, Reactive,
    ToComputed, Value,
};

pub use css::{
    css_structs::{Css, CssGroup},
    tailwind_class::TwClass,
};

pub use dom::{
    attr_value::{AttrValue, CssAttrValue},
    callback::{Callback, Callback1},
    dom_comment::DomComment,
    dom_element::DomElement,
    dom_element_ref::DomElementRef,
    dom_id::DomId,
    dom_node::DomNode,
    dom_text::DomText,
    events::{ClickEvent, DropFileEvent, DropFileItem, KeyDownEvent},
};
pub use dom_macro::{AttrGroup, AttrGroupValue, EmbedDom};
pub use driver_module::{
    api::ApiImport,
    dom_command::DriverDomCommand,
    driver::{Driver, FetchMethod, FetchResult},
    js_value::{
        from_json, to_json, JsJson, JsJsonContext, JsJsonDeserialize, JsJsonObjectBuilder,
        JsJsonSerialize, JsValue, MemoryBlock,
    },
};
use driver_module::{api::CallbackId, init_env::init_env};
pub use fetch::{
    lazy_cache::{self, LazyCache},
    request_builder::{RequestBody, RequestBuilder, RequestResponse},
    resource::Resource,
};
pub use future_box::{FutureBox, FutureBoxSend};
pub use instant::{Instant, InstantType};
pub use long_ptr::LongPtr;
pub use websocket::{WebsocketConnection, WebsocketMessage};

/// Allows to include a static file
///
/// This will place the file along with the rest of generated files. The macro returns a public path to the file with it's hash in name.
pub use vertigo_macro::include_static;

/// Allows to trace additional tailwind class names.
///
/// To use tailwind class name outside of literal tw attribute value, wrap it with `tw!` macro, so it gets traced by tailwind bundler.
///
/// ```rust
/// use vertigo::{dom, tw};
///
/// let my_class = tw!("flex");
///
/// dom! {
///     <div tw={my_class}>
///         <p>"One"</p>
///         <p>"Two"</p>
///     </div>
/// };
/// ```
pub use vertigo_macro::tw;

/// Allows to conveniently clone values into closure.
///
/// ```rust
/// use vertigo::{bind, dom, Value};
///
/// let count = Value::new(0);
///
/// let increment = bind!(count, |_| {
///     count.change(|value| {
///         *value += 1;
///     });
/// });
///
/// dom! {
///     <div>
///         <p>"Counter: " { count }</p>
///         <button on_click={increment}>"+"</button>
///     </div>
/// };
/// ```
///
/// Binding complex names results in last part being accessible inside:
///
/// ```rust
/// use vertigo::bind;
///
/// struct Point {
///     pub x: i32,
///     pub y: i32,
/// }
///
/// let point = Point { x: 1, y: 2 };
///
/// let callback = bind!(point.x, point.y, || {
///     println!("Point: ({x}, {y})");
/// });
/// ```
pub use vertigo_macro::bind;

/// Allows to create an event handler based on provided arguments which is wrapped in Rc
pub use vertigo_macro::bind_rc;

/// Allows to create an event handler based on provided arguments which launches an asynchronous action
pub use vertigo_macro::bind_spawn;

/// Macro for creating `JsJson` from structures and structures from `JsJson`.
///
/// Used for fetching and sending objects over the network.
///
/// Enums representation is compatible with serde's "external tagging" which is the default.
///
/// ```rust
/// #[derive(vertigo::AutoJsJson)]
/// pub struct Post {
///     pub id: i64,
///     pub name: String,
///     pub visible: bool,
/// }
///
/// let post = Post {
///     id: 1,
///     name: "Hello".to_string(),
///     visible: true
/// };
///
/// let js_json = vertigo::to_json(post);
///
/// let post2 = vertigo::from_json::<Post>(js_json);
/// ```
pub use vertigo_macro::AutoJsJson;

/// Macro which transforms a provided function into a component that can be used in [dom!] macro
///
/// ```rust
/// use vertigo::prelude::*;
///
/// #[component]
/// pub fn Header(name: Value<String>) {
///     dom! {
///         <div>"Hello" {name}</div>
///     }
/// }
///
/// let name = Value::new("world".to_string());
///
/// dom! {
///     <div>
///        <Header name={name} />
///     </div>
/// };
/// ```
///
/// ```rust
/// use vertigo::{bind, component, dom, AttrGroup, Value};
///
/// #[component]
/// pub fn Input<'a>(label: &'a str, value: Value<String>, input: AttrGroup) {
///     let on_input = bind!(value, |new_value: String| {
///         value.set(new_value);
///     });
///
///     dom! {
///         <div>
///             {label}
///             <input {value} {on_input} {..input} />
///         </div>
///     }
/// }
///
/// let value = Value::new("world".to_string());
///
/// dom! {
///     <div>
///        <Input label="Hello" {value} input:name="hello_value" />
///     </div>
/// };
/// ```
///
/// Note: [AttrGroup] allows to dynamically pass arguments to some child node.
pub use vertigo_macro::component;

/// Macro that allows to evaluate pseudo-JavaScript expressions.
///
/// Example 1:
///
/// ```rust
/// use vertigo::js;
///
/// let referrer = js!{ document.referrer };
/// ```
///
/// Example 2:
///
/// ```rust
/// # use vertigo::js;
/// let max_y = js!{ window.scrollMaxY };
/// js! { window.scrollTo(0, max_y) };
/// ```
///
/// Can be used with [DomElementRef]:
///
/// ```rust
/// use vertigo::{js, dom_element};
///
/// let node = dom_element! { <input /> };
/// let node_ref = node.get_ref();
/// js! { #node_ref.focus() };
/// ```
///
/// Passing an object as an argument is a little more complicated, but possible:
///
/// ```rust
/// # use vertigo::js;
/// js! {
///     window.scrollTo(
///         vec![
///             ("top", 100000.into()),
///             ("behavior", "smooth".into()),
///         ]
///     )
/// };
/// ```
#[macro_export]
macro_rules! js {
    // Convert `#ref_node.anything` into `#[ref_node] anything` which can be handled by js_inner macro.
    ( #$ident:ident.$expr:expr ) => {
        $crate::js_inner! { #[$ident] $expr }
    };
    // Otherwise be transparent.
    ( $expr:expr ) => {
        $crate::js_inner! { $expr }
    };
}

/// Used internally by [js!] macro.
pub use vertigo_macro::js_inner;

/// Marco that marks an entry point of the app
///
/// Note: Html, head and body tags are required by vertigo to properly take over the DOM
///
/// Note 2: When using external tailwind, make sure the source `tailwind.css` file is in the same directory as usage of this macro.
///
/// ```rust
/// use vertigo::prelude::*;
///
/// #[vertigo::main]
/// fn app() -> DomNode {
///     dom! {
///         <html>
///             <head/>
///             <body>
///                 <div>"Hello world"</div>
///             </body>
///         </html>
///     }
/// }
/// ```
pub use vertigo_macro::main;

// Export log module which can be used in vertigo plugins
pub use log;

/// Allows to create [DomNode] using RSX/rstml (HTML-like) syntax.
///
/// Simple DOM with a param embedded:
///
/// ```rust
/// use vertigo::dom;
///
/// let value = "world";
///
/// dom! {
///     <div>
///         <h3>"Hello " {value} "!"</h3>
///         <p>"Good morning!"</p>
///     </div>
/// };
/// ```
///
/// Mapping and embedding an `Option`:
///
/// ```rust
/// use vertigo::dom;
///
/// let name = "John";
/// let occupation = Some("Lumberjack");
///
/// dom! {
///     <div>
///         <h3>"Hello " {name} "!"</h3>
///         {..occupation.map(|occupation| dom! { <p>"Occupation: " {occupation}</p> })}
///     </div>
/// };
/// ```
///
/// Note the spread operator which utilizes the fact that `Option` is iterable in Rust.
pub use vertigo_macro::dom;

/// Allows to create [DomElement] using HTML tags.
///
/// Unlike [DomNode] generated by the [dom!] macro, it can't generate multiple nodes at top level,
/// but allows to mangle with the outcome a little more, for example using [DomElement::add_child].
pub use vertigo_macro::dom_element;

/// Version of [dom!] macro that additionally emits compiler warning with generated code.
pub use vertigo_macro::dom_debug;

/// Allows to create [Css] styles for usage in [dom!] macro.
///
/// ```rust
/// use vertigo::{css, dom};
///
/// let green_on_red = css!("
///     color: green;
///     background-color: red;
/// ");
///
/// dom! {
///    <div css={green_on_red}>"Tomato stem"</div>
/// };
/// ```
///
/// ```rust
/// use vertigo::{css, Css, dom};
///
/// fn css_menu_item(active: bool) -> Css {
///     let bg_color = if active { "lightblue" } else { "lightgreen" };
///
///     css! {"
///         cursor: pointer;
///         background-color: {bg_color};
///
///         :hover {
///             text-decoration: underline;
///         }
///     "}
/// }
///
/// dom! {
///     <a css={css_menu_item(true)}>"Active item"</a>
///     <a css={css_menu_item(false)}>"Inactive item"</a>
/// };
/// ```
///
/// See [tooltip demo](https://github.com/vertigo-web/vertigo/blob/master/demo/app/src/app/styling/tooltip.rs) for more complex example.
pub use vertigo_macro::css;

/// Constructs a CSS block that can be manually pushed into existing [Css] styles instance.
///
/// ```rust
/// use vertigo::{css, css_block};
///
/// let mut green_style = css!("
///     color: green;
/// ");
///
/// green_style.push_str(
///     css_block! { "
///         font-style: italic;
///    " }
/// );
/// ```
pub use vertigo_macro::css_block;

/// Wraps a function generating a resource out of parameters, and creates a store.
///
/// Accessing the store from different locations uses always the same store
/// as data is kept using [LocalKey](std::thread::LocalKey).
///
/// ```rust
/// use vertigo::{AutoJsJson, LazyCache, RequestBuilder, store};
///
/// #[derive(AutoJsJson)]
/// struct CommentModel {
///     id: i32,
///     name: String,
/// }
///
/// #[store]
/// fn get_post(post_id: &String) -> LazyCache<Vec<CommentModel>> {
///     RequestBuilder
///         ::get(format!("https://jsonplaceholder.typicode.com/posts/{post_id}/comments"))
///         .ttl_minutes(10)
///         .lazy_cache(|status, body| {
///             if status == 200 {
///                 Some(body.into::<Vec<CommentModel>>())
///             } else {
///                 None
///             }
///         })
/// }
pub use vertigo_macro::store;

pub mod html_entities;

pub struct DriverConstruct {
    driver: Driver,
    subscription: ValueMut<Option<DomNode>>,
}

impl DriverConstruct {
    fn new() -> DriverConstruct {
        let driver = Driver::default();

        DriverConstruct {
            driver,
            subscription: ValueMut::new(None),
        }
    }

    fn set_root(&self, root_view: DomNode) {
        self.subscription.set(Some(root_view));
    }
}

extern crate self as vertigo;

#[store]
fn get_driver_browser() -> Rc<DriverConstruct> {
    Rc::new(DriverConstruct::new())
}

/// Starting point of the app (used by [main] macro, which is preferred)
pub fn start_app(init_app: fn() -> DomNode) {
    let state = get_driver_browser();

    init_env();

    let root_view = init_app();

    api_fetch_event().on_fetch_start.trigger(());

    state.set_root(root_view);

    api_fetch_event().on_fetch_stop.trigger(());
    get_driver().inner.dom.flush_dom_changes();
}

#[doc(hidden)]
#[no_mangle]
pub fn vertigo_export_handle_url(url_ptr: u64) -> u64 {
    let url_ptr = LongPtr::from(url_ptr);
    let url = api_arguments().get_by_long_ptr(url_ptr);

    let JsValue::String(url) = url else {
        panic!("string expected");
    };

    let response = api_server_handler().handler(&url);
    response.to_ptr_long().get_long_ptr()
}

/// Getter for [Driver] singleton.
///
/// ```rust
/// use vertigo::get_driver;
///
/// let number = get_driver().get_random(1, 10);
/// ```
pub fn get_driver() -> Driver {
    get_driver_browser().driver
}

/// Do bunch of operations on dependency graph without triggering anything in between.
pub fn transaction<R, F: FnOnce(&Context) -> R>(f: F) -> R {
    get_driver().transaction(f)
}

pub mod prelude {
    pub use crate::{bind, component, css, dom, Computed, Css, DomNode, ToComputed, Value};
}

//------------------------------------------------------------------------------------------------------------------
// Internals below
//------------------------------------------------------------------------------------------------------------------

pub use driver_module::driver::{
    VERTIGO_MOUNT_POINT_PLACEHOLDER, VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER,
};

use crate::driver_module::api::{
    api_arguments, api_callbacks, api_fetch_event, api_server_handler,
};

// Methods for memory allocation

#[doc(hidden)]
#[no_mangle]
pub fn vertigo_export_alloc_block(size: u32) -> u64 {
    api_arguments().alloc(size).get_long_ptr()
}

#[doc(hidden)]
#[no_mangle]
pub fn vertigo_export_free_block(long_ptr: u64) {
    let long_ptr = LongPtr::from(long_ptr);
    api_arguments().free(long_ptr);
}

// Callbacks gateways

#[doc(hidden)]
#[no_mangle]
pub fn vertigo_export_wasm_callback(callback_id: u64, value_long_ptr: u64) -> u64 {
    let value_long_ptr = LongPtr::from(value_long_ptr);

    let value = api_arguments().get_by_long_ptr(value_long_ptr);
    let callback_id = CallbackId::from_u64(callback_id);

    let mut result = JsValue::Undefined;

    get_driver().transaction(|_| {
        result = api_callbacks().call(callback_id, value);
    });

    result.to_ptr_long().get_long_ptr()
}
