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
//! ```rust,no_run
//! use std::cmp::PartialEq;
//! use vertigo::{Computed, Driver, VDomElement, Value, html, css_fn};
//!
//! #[derive(PartialEq)]
//! pub struct State {
//!     driver: Driver,
//!
//!     pub message: Value<String>,
//! }
//!
//! impl State {
//!     pub fn new(driver: &Driver) -> Computed<State> {
//!         let state = State {
//!             driver: driver.clone(),
//!             message: driver.new_value("Hello world".to_string()),
//!         };
//!
//!         driver.new_computed_from(state)
//!     }
//! }
//!
//! css_fn! { main_div, "
//!     color: darkblue;
//! " }
//!
//! pub fn render(app_state: &Computed<State>) -> VDomElement {
//!     let state = app_state.get_value();
//!
//!     html! {
//!         <div css={main_div()}>
//!             "Message to the world: "
//!             {state.message.get_value()}
//!         </div>
//!     }
//! }
//! ```
//!
//! More description soon! For now, to get started you may consider looking
//! at the [Tutorial](https://github.com/vertigo-web/vertigo/blob/master/tutorial.md).

#![deny(rust_2018_idioms)]
#![feature(trait_alias)]
#![feature(try_trait_v2)] // https://github.com/rust-lang/rust/issues/84277

mod app;
mod computed;
mod css;
mod driver;
mod driver_refs;
mod fetch;
mod html_macro;
mod instant;
pub mod router;
pub mod utils;
mod virtualdom;
mod websocket;

pub use computed::{AutoMap, Computed, Dependencies, Value, struct_mut, Client};
pub use driver::{Driver, FetchResult};
pub use fetch::{
    fetch_builder::FetchBuilder,
    lazy_cache,
    lazy_cache::LazyCache,
    request_builder::{ListRequestTrait, RequestBuilder, RequestResponse, SingleRequestTrait},
    resource::Resource,
};
pub use html_macro::Embed;
pub use instant::{Instant, InstantType};
pub use utils::DropResource;
pub use virtualdom::models::{
    css::{Css, CssGroup},
    vdom_element::{KeyDownEvent, VDomElement},
};
pub use websocket::{WebsocketConnection, WebsocketMessage};
pub use app::start_app;

pub mod dev {
    pub use super::app::start_app;
    pub use super::driver::{DriverTrait, EventCallback, FetchMethod};
    pub use super::driver_refs::RefsContext;
    pub use super::virtualdom::models::{
        node_attr,
        realdom_id::RealDomId,
        vdom_component::VDomComponent,
        vdom_node::VDomNode,
        vdom_refs::{NodeRefs, NodeRefsItem},
        vdom_text::VDomText,
    };
    pub use super::websocket::WebsocketMessageDriver;
}

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
/// ```rust,no_run
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
/// ```rust,no_run
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
/// ```rust,no_run
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
