mod app;
pub mod computed;
mod css;
mod fetch_builder;
mod instant;
mod driver;
pub mod router;
pub mod utils;
mod virtualdom;

pub use driver::DomDriver;
pub use driver::DomDriverTrait;
pub use driver::EventCallback;
pub use driver::FetchResult;
pub use driver::FetchMethod;

pub use virtualdom::models::realdom_id::RealDomId;
pub use virtualdom::models::vdom_component::VDomComponent;
pub use virtualdom::models::vdom_element::{VDomElement, KeyDownEvent};
pub use virtualdom::models::vdom_refs::{NodeRefs, NodeRefsItem, NodeRefsItemTrait};
pub use virtualdom::models::vdom_text::VDomText;
pub use virtualdom::models::vdom_node::VDomNode;
pub use virtualdom::models::css::{Css, CssGroup};
pub use virtualdom::models::node_attr;

pub use instant::{Instant, InstantType};

// Export log module which can be used in vertigo plugins
pub use log;

pub use app::start_app;
