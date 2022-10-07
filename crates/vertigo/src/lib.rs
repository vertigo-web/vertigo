//! Vertigo is a library for building reactive web components.
//!
//! It mainly consists of three parts:
//!
//! * **Reactive dependencies** - A graph of values and clients (micro-subscriptions) that can automatically compute what to refresh after one value change
//! * **Real DOM** - No intermediate Virtual DOM mechanism is necessary
//! * **HTML/CSS macros** - Allows to construct Real DOM nodes using HTML and CSS
//!
//! ## Example
//!
//! ```rust
//! use vertigo::{DomElement, Value, dom, css_fn};
//!
//! pub struct State {
//!     pub message: Value<String>,
//! }
//!
//! impl State {
//!     pub fn component() -> DomElement {
//!         let state = State {
//!             message: Value::new("Hello world".to_string()),
//!         };
//!
//!         render(state)
//!     }
//! }
//!
//! css_fn! { main_div, "
//!     color: darkblue;
//! " }
//!
//! fn render(state: State) -> DomElement {
//!     dom! {
//!         <div css={main_div()}>
//!             "Message to the world: "
//!             { state.message }
//!         </div>
//!     }
//! }
//! ```
//!
//! More description soon! For now, to get started you may consider looking
//! at the [Tutorial](https://github.com/vertigo-web/vertigo/blob/master/tutorial.md).

#![deny(rust_2018_idioms)]
#![feature(try_trait_v2)] // https://github.com/rust-lang/rust/issues/84277
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::new_without_default)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::non_send_fields_in_send_ty)]
#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::from_over_into)]

mod computed;
mod css;
mod fetch;
mod html_macro;
mod instant;
pub mod router;
mod dom;
mod websocket;
mod future_box;
mod bind;
mod driver_module;
mod dom_value;
mod dom_list;

pub use computed::{AutoMap, Computed, Dependencies, Value, struct_mut, Client, GraphId, DropResource};
pub use driver_module::driver::{Driver};
pub use driver_module::driver::{FetchResult};
pub use fetch::{
    fetch_builder::FetchBuilder,
    lazy_cache,
    lazy_cache::LazyCache,
    request_builder::{ListRequestTrait, RequestBuilder, RequestResponse, SingleRequestTrait},
    resource::Resource,
};
pub use html_macro::EmbedDom;
pub use instant::{Instant, InstantType};
pub use dom::{
    css::{Css, CssGroup},
    dom_element::DomElement,
    dom_text::DomText,
    dom_comment::DomComment,
    dom_node::DomNode,
    dom_node::DomNodeFragment,
    dom_comment_create::DomCommentCreate,
};
pub use crate::dom_list::ListRendered;
pub use dom::types::{
    KeyDownEvent, DropFileEvent, DropFileItem
};
pub use websocket::{WebsocketConnection, WebsocketMessage};
pub use future_box::{FutureBoxSend, FutureBox};
pub use bind::{bind, bind2, bind3, bind4, Bind1, Bind2, Bind3, Bind4};
pub use driver_module::driver::{FetchMethod};
pub use dom::{
    dom_id::DomId,
};
pub use websocket::WebsocketMessageDriver;
pub use crate::fetch::pinboxfut::PinBoxFuture;

pub use computed::context::{Context};
pub use crate::driver_module::api::ApiImport;
pub use crate::driver_module::js_value::js_value_struct::JsValue;

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

mod external_api;
use external_api::{DRIVER_BROWSER, DriverConstruct};

fn get_driver_state<R: Default, F: FnOnce(&DriverConstruct) -> R>(label: &'static str, onec: F) -> R {
    match DRIVER_BROWSER.try_with(onec) {
        Ok(value) => value,
        Err(_) => {
            println!("error access {label}");
            R::default()
        }
    }
}

//------------------------------------------------------------------------------------------------------------------
// methods for memory allocation
//------------------------------------------------------------------------------------------------------------------

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

//------------------------------------------------------------------------------------------------------------------

#[no_mangle]
pub fn interval_run_callback(callback_id: u32) {
    get_driver_state("interval_run_callback", |state| state.driver.export_interval_run_callback(callback_id));
}

#[no_mangle]
pub fn timeout_run_callback(callback_id: u32) {
    get_driver_state("timeout_run_callback", |state| state.driver.export_timeout_run_callback(callback_id));
}

#[no_mangle]
pub fn hashrouter_hashchange_callback(list_id: u32) {
    get_driver_state("hashrouter_hashchange_callback", |state| state.driver.export_hashrouter_hashchange_callback(list_id));
}

#[no_mangle]
pub fn fetch_callback(params_id: u32) {
    get_driver_state("fetch_callback", |state| state.driver.export_fetch_callback(params_id));
}

#[no_mangle]
pub fn websocket_callback_socket(callback_id: u32) {
    get_driver_state("websocket_callback_socket", |state| state.driver.export_websocket_callback_socket(callback_id));
}

#[no_mangle]
pub fn websocket_callback_message(callback_id: u32) {
    get_driver_state("websocket_callback_message", |state| state.driver.export_websocket_callback_message(callback_id));
}

#[no_mangle]
pub fn websocket_callback_close(callback_id: u32) {
    get_driver_state("websocket_callback_close", |state| state.driver.export_websocket_callback_close(callback_id));
}

#[no_mangle]
pub fn export_dom_callback(callback_id: u64, value_ptr: u32) -> u64 {
    get_driver_state("export_dom_callback", |state| {
        let (ptr, size) = state.driver.export_dom_callback(callback_id, value_ptr);

        let ptr = ptr as u64;
        let size = size as u64;

        (ptr << 32) + size
    })
}

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

pub fn get_driver() -> Driver {
    DRIVER_BROWSER.with(|state| {
        state.driver.clone()
    })
}

pub fn transaction<F: FnOnce(&Context)>(f: F) {
    get_driver().transaction(f)
}

