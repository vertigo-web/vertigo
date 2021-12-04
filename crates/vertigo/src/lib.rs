#![feature(trait_alias)]
#![feature(try_trait_v2)] // https://github.com/rust-lang/rust/issues/84277

mod app;
pub mod computed;
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

pub use app::start_app;
pub use computed::{AutoMap, Computed, Value};
pub use driver::{Driver, DriverTrait, EventCallback, FetchResult, FetchMethod};
pub use driver_refs::RefsContext;
pub use fetch::{
    request_builder::{SingleRequestTrait, ListRequestTrait},
    resource::Resource,
    lazy_cache,
    lazy_cache::LazyCache,
};
pub use html_macro::Embed;
pub use instant::{Instant, InstantType};
pub use utils::DropResource;
pub use vertigo_macro::{html, css, css_block};
pub use virtualdom::models::{
    realdom_id::RealDomId,
    vdom_component::VDomComponent,
    vdom_element::{VDomElement, KeyDownEvent},
    vdom_refs::{NodeRefs, NodeRefsItem},
    vdom_text::VDomText,
    vdom_node::VDomNode,
    css::{Css, CssGroup},
    node_attr,
};
pub use websocket::{WebcocketMessageDriver, WebcocketMessage, WebcocketConnection};

#[cfg(feature = "serde_request")]
pub use vertigo_macro::{SerdeRequest, SerdeSingleRequest, SerdeListRequest};
#[cfg(feature = "serde_request")]
pub use serde_json;

// Export log module which can be used in vertigo plugins
pub use log;
