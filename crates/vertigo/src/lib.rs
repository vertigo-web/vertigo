#![feature(trait_alias)]
#![feature(try_trait_v2)]               //https://github.com/rust-lang/rust/issues/84277

mod app;
pub mod computed;
mod css;
mod fetch;
mod html_macro;
mod instant;
mod driver;
mod driver_refs;
pub mod router;
pub mod utils;
mod virtualdom;
mod websocket;

pub use driver::Driver;
pub use driver::DriverTrait;
pub use driver::EventCallback;
pub use driver::FetchResult;
pub use driver::FetchMethod;

pub use driver_refs::RefsContext;
pub use virtualdom::models::realdom_id::RealDomId;
pub use virtualdom::models::vdom_component::VDomComponent;
pub use virtualdom::models::vdom_element::{VDomElement, KeyDownEvent};
pub use virtualdom::models::vdom_refs::{NodeRefs, NodeRefsItem};
pub use virtualdom::models::vdom_text::VDomText;
pub use virtualdom::models::vdom_node::VDomNode;
pub use virtualdom::models::css::{Css, CssGroup};
pub use virtualdom::models::node_attr;

pub use instant::{Instant, InstantType};

pub use html_macro::Embed;
pub use vertigo_macro::{html, css, css_block};

#[cfg(feature = "serde_request")]
pub use vertigo_macro::{SerdeRequest, SerdeSingleRequest, SerdeListRequest};
#[cfg(feature = "serde_request")]
pub use serde_json;

// Export log module which can be used in vertigo plugins
pub use log;

pub use app::start_app;

pub use fetch::request_builder::{SingleRequestTrait, ListRequestTrait};
pub use fetch::resource::Resource;
pub use fetch::lazy_cache;
pub use fetch::lazy_cache::LazyCache;
pub use websocket::{WebcocketMessageDriver, WebcocketMessage, WebcocketConnection};
