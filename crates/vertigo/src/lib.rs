//! Vertigo is a library for building reactive web components.
//!
//! It mainly consists of three parts:
//!
//! * **Reactive dependencies** - A graph of values and clients (micro-subscriptions)
//!   that can automatically compute what to refresh after one or more change(s)
//! * **Real DOM operations** - No intermediate Virtual DOM mechanism is necessary
//! * **HTML/CSS macros** - Allows to construct Real DOM nodes using HTML and CSS
//!
//! ## Example 1
//!
//! ```rust
//! use vertigo::{dom, DomElement, Value, bind, start_app};
//!
//! pub fn render(count: Value<i32>) -> DomElement {
//!     let increment = bind(&count).call(|context, count| {
//!         count.set(count.get(context) + 1);
//!     });
//!
//!     let decrement = bind(&count).call(|context, count| {
//!         count.set(count.get(context) - 1);
//!     });
//!
//!     dom! {
//!         <div>
//!             <p>"Counter: " { count }</p>
//!             <button on_click={decrement}>"-"</button>
//!             <button on_click={increment}>"+"</button>
//!         </div>
//!     }
//! }
//!
//! #[no_mangle]
//! pub fn start_application() {
//!     start_app(|| -> DomElement {
//!         let count = Value::new(0);
//!         render(count)
//!     });
//! }
//! ```
//!
//! ## Example 2
//!
//! ```rust
//! use vertigo::{DomElement, Value, dom, css_fn};
//!
//! pub struct MyMessage {
//!     pub message: Value<String>,
//! }
//!
//! impl MyMessage {
//!     pub fn mount(self) -> DomElement {
//!         dom! {
//!             <p>
//!                 "Message to the world: "
//!                 { self.message }
//!             </p>
//!         }
//!     }
//! }
//!
//! css_fn! { main_div, "
//!     color: darkblue;
//! " }
//!
//! fn render() -> DomElement {
//!     let message = Value::new("Hello world!".to_string());
//!
//!     dom! {
//!         <div css={main_div()}>
//!             <MyMessage message={message} />
//!         </div>
//!     }
//! }
//! ```
//!
//! To get started you may consider looking at the
//! [Tutorial](https://github.com/vertigo-web/vertigo/blob/master/tutorial.md).

#![deny(rust_2018_idioms)]
#![feature(try_trait_v2)] // https://github.com/rust-lang/rust/issues/84277

mod bind;
mod computed;
mod css;
mod dom_list;
mod dom_value;
mod dom;
mod driver_module;
mod fetch;
mod future_box;
mod html_macro;
pub mod inspect;
mod instant;
pub mod router;
mod websocket;

pub use bind::{bind, bind2, bind3, bind4, Bind1, Bind2, Bind3, Bind4};
pub use computed::{
    AutoMap, Client, Computed, context::Context, Dependencies, DropResource, GraphId, struct_mut, Value
};
pub use dom::{
    css::{Css, CssGroup},
    dom_id::DomId,
    dom_comment::DomComment,
    dom_comment_create::DomCommentCreate,
    dom_element::DomElement,
    dom_node::{DomNode, DomNodeFragment},
    dom_text::DomText,
    types::{KeyDownEvent, DropFileEvent, DropFileItem},
};
pub use dom_list::ListRendered;
pub use driver_module::{
    api::ApiImport,
    driver::{Driver, FetchResult, FetchMethod},
    js_value::js_value_struct::JsValue,
    modules::dom::DriverDomCommand,
};
pub use fetch::{
    fetch_builder::FetchBuilder,
    lazy_cache::{self, LazyCache},
    pinboxfut::PinBoxFuture,
    request_builder::{ListRequestTrait, RequestBuilder, RequestResponse, SingleRequestTrait},
    resource::Resource,
};
pub use future_box::{FutureBoxSend, FutureBox};
pub use html_macro::{
    EmbedDom, clone_if_ref
};
pub use instant::{
    Instant, InstantType
};
pub use websocket::{WebsocketConnection, WebsocketMessageDriver, WebsocketMessage};


#[cfg(feature = "serde_request")]
/// Implements [SingleRequestTrait] using serde (needs `serde_request` feature).
pub use vertigo_macro::SerdeListRequest;

#[cfg(feature = "serde_request")]
/// Implements both [SingleRequestTrait] and [ListRequestTrait] using serde (needs `serde_request` feature).
pub use vertigo_macro::SerdeRequest;

#[cfg(feature = "serde_request")]
/// Implements [ListRequestTrait] using serde (needs `serde_request` feature).
pub use vertigo_macro::SerdeSingleRequest;

#[cfg(feature = "serde_request")]
pub use serde_json;

// Export log module which can be used in vertigo plugins
pub use log;

/// Allows to create DomElement using HTML tags.
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
pub use vertigo_macro::dom;

/// Version of `dom!` macro that additionally emits compiler warning with generated code.
pub use vertigo_macro::dom_debug;

/// Allows to create Css styles for virtual DOM.
///
/// ```rust
/// use vertigo::{Css, css};
///
/// fn green_on_red() -> Css {
///     css! { "
///         color: green;
///         background-color: red;
///     " }
/// }
/// ```
pub use vertigo_macro::css;

/// Constructs a CSS block that can be manually pushed into existing Css styles instance.
///
/// ```rust
/// use vertigo::{css_fn, css_block};
///
/// css_fn! { green, "
///     color: green;
/// " }
///
/// let mut green_style = green();
///
/// green_style.push_str(
///     css_block! { "
///         font-style: italic;
///    " }
/// );
/// ```
pub use vertigo_macro::css_block;

/// Starting point of the app.
pub fn start_app(get_component: impl FnOnce() -> DomElement) {
    get_driver_state("start_app", |state| {
        state.driver.init_env();
        let app = get_component();

        let root = DomElement::create_with_id(DomId::root());
        root.add_child(app);
        state.set_root(root);

        get_driver().inner.dom.flush_dom_changes();
    });
}

/// Getter for Driver singleton.
pub fn get_driver() -> Driver {
    DRIVER_BROWSER.with(|state| {
        state.driver.clone()
    })
}

/// Do bunch of operations without triggering anything in between.
pub fn transaction<F: FnOnce(&Context)>(f: F) {
    get_driver().transaction(f)
}

//------------------------------------------------------------------------------------------------------------------
// Internals below
//------------------------------------------------------------------------------------------------------------------

mod external_api;
use external_api::{DRIVER_BROWSER, DriverConstruct};

fn get_driver_state<R: Default, F: FnOnce(&DriverConstruct) -> R>(label: &'static str, once: F) -> R {
    match DRIVER_BROWSER.try_with(once) {
        Ok(value) => value,
        Err(_) => {
            println!("error access {label}");
            R::default()
        }
    }
}

// Methods for memory allocation

#[no_mangle]
pub fn alloc(size: u32) -> u32 {
    get_driver_state("alloc", |state| {
        state.driver.inner.api.arguments.alloc(size)
    })
}

#[no_mangle]
pub fn free(pointer: u32) {
    get_driver_state("free", |state| {
        state.driver.inner.api.arguments.free(pointer);
    })
}

// Callbacks gateways

#[no_mangle]
pub fn wasm_callback(callback_id: u64, value_ptr: u32) -> u64 {
    get_driver_state("export_dom_callback", |state| {
        let (ptr, size) = state.driver.wasm_callback(callback_id, value_ptr);

        let ptr = ptr as u64;
        let size = size as u64;

        (ptr << 32) + size
    })
}
