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
//! use vertigo::{Computed, VDomElement, VDomComponent, Value, html, css_fn};
//!
//! pub struct State {
//!     pub message: Value<String>,
//! }
//!
//! impl State {
//!     pub fn component() -> VDomComponent {
//!         let state = State {
//!             message: Value::new("Hello world".to_string()),
//!         };
//!
//!         VDomComponent::from(state, render)
//!     }
//! }
//!
//! css_fn! { main_div, "
//!     color: darkblue;
//! " }
//!
//! fn render(state: &State) -> VDomElement {
//!     html! {
//!         <div css={main_div()}>
//!             "Message to the world: "
//!             {state.message.get()}
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

mod app;
mod computed;
mod css;
mod driver_refs;
mod fetch;
mod html_macro;
mod instant;
pub mod router;
mod virtualdom;
mod websocket;
mod future_box;
mod bind;
mod driver_module;

pub use computed::{AutoMap, Computed, Dependencies, Value, struct_mut, Client, GraphId, DropResource};
pub use driver_module::driver_browser::{Driver};
pub use driver_module::driver_browser::{FetchResult};
use driver_module::stack::ListId;
pub use fetch::{
    fetch_builder::FetchBuilder,
    lazy_cache,
    lazy_cache::LazyCache,
    request_builder::{ListRequestTrait, RequestBuilder, RequestResponse, SingleRequestTrait},
    resource::Resource,
};
pub use html_macro::Embed;
pub use instant::{Instant, InstantType};
pub use virtualdom::models::{
    css::{Css, CssGroup},
    vdom_element::{KeyDownEvent, DropFileEvent, DropFileItem, VDomElement},
    vdom_component::VDomComponent,
    vdom_node::VDomNode,
};
pub use websocket::{WebsocketConnection, WebsocketMessage};
pub use future_box::{FutureBoxSend, FutureBox};
pub use bind::bind;
pub mod dev {
    pub use super::driver_module::driver_browser::{EventCallback, FetchMethod};
    pub use super::driver_refs::RefsContext;
    pub use super::virtualdom::models::{
        node_attr,
        realdom_id::RealDomId,
        vdom_node::VDomNode,
        vdom_refs::{NodeRefs, NodeRefsItem},
        vdom_text::VDomText,
    };
    pub use super::websocket::WebsocketMessageDriver;
    pub use crate::fetch::pinboxfut::PinBoxFuture;
}

pub use crate::driver_module::api::ApiImport;

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

/// Allows to create VDomElement using HTML tags.
///
/// ```rust
/// use vertigo::html;
///
/// let value = "world";
///
/// html! {
///     <div>
///         <h3>"Hello " {value} "!"</h3>
///         <p>"Good morning!"</p>
///     </div>
/// };
/// ```
pub use vertigo_macro::html;

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
pub fn arguments_new_list() -> u32 {
    DRIVER_BROWSER.with(|state| {
        state.driver.driver.api.arguments.new_list().to_u32()
    })
}

#[no_mangle]
pub fn arguments_debug(list_id: u32) {
    let list_id = ListId(list_id);
    DRIVER_BROWSER.with(|state| {
        state.driver.driver.api.arguments.debug(list_id)
    })
}

#[no_mangle]
pub fn arguments_push_string_empty(list_id: u32) {
    let list_id = ListId(list_id);
    DRIVER_BROWSER.with(|state| {
        state.driver.driver.api.arguments.push_string_empty(list_id);
    })
}

#[no_mangle]
pub fn arguments_push_string_alloc(list_id: u32, size: u32) -> u32 {
    let list_id = ListId(list_id);
    DRIVER_BROWSER.with(|state| {
        state.driver.driver.api.arguments.push_string_alloc(list_id, size).to_u32()
    })
}

#[no_mangle]
pub fn arguments_push_buffer_alloc(list_id: u32, size: u32) -> u32 {
    let list_id = ListId(list_id);
    DRIVER_BROWSER.with(|state| {
        state.driver.driver.api.arguments.push_buffer_alloc(list_id, size).to_u32()
    })
}

#[no_mangle]
pub fn arguments_push_u32(list_id: u32, value: u32) {
    let list_id = ListId(list_id);
    DRIVER_BROWSER.with(|state| {
        state.driver.driver.api.arguments.push_u32(list_id, value)
    })
}

#[no_mangle]
pub fn arguments_push_i32(list_id: u32, value: i32) {
    let list_id = ListId(list_id);
    DRIVER_BROWSER.with(|state| {
        state.driver.driver.api.arguments.push_i32(list_id, value)
    })
}

#[no_mangle]
pub fn arguments_push_u64(list_id: u32, value: u64) {
    let list_id = ListId(list_id);
    DRIVER_BROWSER.with(|state| {
        state.driver.driver.api.arguments.push_u64(list_id, value)
    })
}

#[no_mangle]
pub fn arguments_push_i64(list_id: u32, value: i64) {
    let list_id = ListId(list_id);
    DRIVER_BROWSER.with(|state| {
        state.driver.driver.api.arguments.push_i64(list_id, value)
    })
}

#[no_mangle]
pub fn arguments_push_true(list_id: u32) {
    let list_id = ListId(list_id);
    DRIVER_BROWSER.with(|state| {
        state.driver.driver.api.arguments.push_true(list_id)
    })
}

#[no_mangle]
pub fn arguments_push_false(list_id: u32) {
    let list_id = ListId(list_id);
    DRIVER_BROWSER.with(|state| {
        state.driver.driver.api.arguments.push_false(list_id)
    })
}

#[no_mangle]
pub fn arguments_push_null(list_id: u32) {
    let list_id = ListId(list_id);
    DRIVER_BROWSER.with(|state| {
        state.driver.driver.api.arguments.push_null(list_id)
    })
}

#[no_mangle]
pub fn arguments_push_sublist(id: u32, sub_params_id: u32) {
    let id = ListId(id);
    let sub_params_id = ListId(sub_params_id);
    DRIVER_BROWSER.with(|state| {
        state.driver.driver.api.arguments.push_list(id, sub_params_id);
    })
}

#[no_mangle]
pub fn arguments_freeze(list_id: u32) {
    let list_id = ListId(list_id);
    DRIVER_BROWSER.with(|state| {
        state.driver.driver.api.arguments.freeze(list_id)
    })
}

//------------------------------------------------------------------------------------------------------------------

#[no_mangle]
pub fn interval_run_callback(callback_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver.driver.export_interval_run_callback(callback_id));
}

#[no_mangle]
pub fn timeout_run_callback(callback_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver.driver.export_timeout_run_callback(callback_id));
}

#[no_mangle]
pub fn hashrouter_hashchange_callback(list_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver.driver.export_hashrouter_hashchange_callback(list_id));
}

#[no_mangle]
pub fn fetch_callback(params_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver.driver.export_fetch_callback(params_id));
}

#[no_mangle]
pub fn websocket_callback_socket(callback_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver.driver.export_websocket_callback_socket(callback_id));
}

#[no_mangle]
pub fn websocket_callback_message(callback_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver.driver.export_websocket_callback_message(callback_id));
}

#[no_mangle]
pub fn websocket_callback_close(callback_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver.driver.export_websocket_callback_close(callback_id));
}

#[no_mangle]
pub fn dom_keydown(params_id: u32) -> u32 {
    DRIVER_BROWSER.with(|state|
        state.driver.driver.export_dom_keydown(params_id)
    )
}

#[no_mangle]
pub fn dom_oninput(params_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver.driver.export_dom_oninput(params_id));
}

#[no_mangle]
pub fn dom_mouseover(dom_id: u64) {
    DRIVER_BROWSER.with(|state| state.driver.driver.export_dom_mouseover(dom_id));
}

#[no_mangle]
pub fn dom_mousedown(dom_id: u64) {
    DRIVER_BROWSER.with(|state| state.driver.driver.export_dom_mousedown(dom_id));
}

#[no_mangle]
pub fn dom_ondropfile(params_id: u32) {
    DRIVER_BROWSER.with(|state| state.driver.driver.export_dom_ondropfile(params_id));
}

pub fn start_app(get_component: impl FnOnce() -> VDomComponent) {
    DRIVER_BROWSER.with(|state| {
        state.driver.driver.init_env();
        let component = get_component();
        let driver = state.driver.clone();

        let client = crate::app::start_app(driver, component);

        let mut inner = state.subscription.borrow_mut();
        *inner = Some(client);
    });
}

pub(crate) fn get_dependencies() -> Dependencies {
    DRIVER_BROWSER.with(|state| {
        state.driver.get_dependencies()
    })
}

pub(crate) fn external_connections_refresh() {                      //TODO - move somewhere ?
    DRIVER_BROWSER.with(|state| {
        state.driver.external_connections_refresh();
    });
}

pub fn get_driver() -> Driver {                                     //TODO - move somewhere ?
    DRIVER_BROWSER.with(|state| {
        state.driver.clone()
    })
}


