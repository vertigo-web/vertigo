//! Vertigo is a library for building reactive web components.
//!
//! It mainly consists of three parts:
//!
//! * **Virtual DOM** - Lightweight representation of JavaScript DOM that can be used to optimally update real DOM
//! * **Reactive dependencies** - A graph of values and clients that can automatically compute what to refresh after one value change
//! * **HTML/CSS macros** - Allows to construct Virtual DOM nodes using HTML and CSS
//!
//! ## Example
//!
//! ```rust
//! use vertigo::{Computed, DomElement, Value, dom, css_fn};
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
//!             <text computed={&state.message} />
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
};
pub use dom::types::{
    KeyDownEvent, DropFileEvent, DropFileItem
};
pub use websocket::{WebsocketConnection, WebsocketMessage};
pub use future_box::{FutureBoxSend, FutureBox};
pub use bind::bind;
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
use external_api::DRIVER_BROWSER;

//------------------------------------------------------------------------------------------------------------------
// methods for memory allocation
//------------------------------------------------------------------------------------------------------------------

#[no_mangle]
pub fn alloc(size: u32) -> u32 {
    DRIVER_BROWSER.with(|state| {
        state.driver.driver_inner.api.arguments.alloc(size)
    })
}

//------------------------------------------------------------------------------------------------------------------

#[no_mangle]
pub fn interval_run_callback(callback_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver.export_interval_run_callback(callback_id));
}

#[no_mangle]
pub fn timeout_run_callback(callback_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver.export_timeout_run_callback(callback_id));
}

#[no_mangle]
pub fn hashrouter_hashchange_callback(list_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver.export_hashrouter_hashchange_callback(list_id));
}

#[no_mangle]
pub fn fetch_callback(params_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver.export_fetch_callback(params_id));
}

#[no_mangle]
pub fn websocket_callback_socket(callback_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver.export_websocket_callback_socket(callback_id));
}

#[no_mangle]
pub fn websocket_callback_message(callback_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver.export_websocket_callback_message(callback_id));
}

#[no_mangle]
pub fn websocket_callback_close(callback_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver.export_websocket_callback_close(callback_id));
}

#[no_mangle]
pub fn dom_keydown(params_id: u32) -> u32 {
    DRIVER_BROWSER.with(|state|
        state.driver.export_dom_keydown(params_id)
    )
}

#[no_mangle]
pub fn dom_oninput(params_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver.export_dom_oninput(params_id));
}

#[no_mangle]
pub fn dom_mouseover(dom_id: u64) {
    DRIVER_BROWSER.with(|state| state.driver.export_dom_mouseover(dom_id));
}

#[no_mangle]
pub fn dom_mousedown(dom_id: u64) {
    DRIVER_BROWSER.with(|state| state.driver.export_dom_mousedown(dom_id));
}

#[no_mangle]
pub fn dom_ondropfile(params_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver.export_dom_ondropfile(params_id));
}

/// Starting point of the app.
pub fn start_app(get_component: impl FnOnce() -> DomElement) {
    DRIVER_BROWSER.with(|state| {
        state.driver.init_env();
        let app = get_component();

        let root = DomElement::create_with_id(DomId::root());
        root.add_child(app);

        let mut inner = state.subscription.borrow_mut();
        *inner = Some(root);
        drop(inner);

        get_driver().flush_update();
    });
}

pub(crate) fn get_dependencies() -> Dependencies {
    DRIVER_BROWSER.with(|state| {
        state.driver.get_dependencies()
    })
}

pub(crate) fn external_connections_refresh() {                      //TODO - move somewhere ?
    DRIVER_BROWSER.with(|state| {
        state.driver.get_dependencies().external_connections_refresh();
    });
}

pub fn get_driver() -> Driver {                                     //TODO - move somewhere ?
    DRIVER_BROWSER.with(|state| {
        state.driver.clone()
    })
}

pub fn transaction<F: FnOnce(&Context)>(f: F) {
    get_driver().transaction(f)
}

