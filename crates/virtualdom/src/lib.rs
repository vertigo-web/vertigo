#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub mod computed;
mod vdom;
mod driver;
mod app;

pub use driver::DomDriver;
pub use driver::DomDriverTrait;
pub use driver::FetchMethod;
pub use driver::FetchError;
pub use driver::EventCallback;

pub use vdom::models::RealDomId::RealDomId;
pub use vdom::models::VDomComponent::VDomComponent;
pub use vdom::models::VDomNode::VDomNode;
pub use vdom::models::Css::Css;
pub use vdom::models::Css::CssFrames;
pub use vdom::models::NodeAttr;

pub use app::App;