#![feature(trait_alias)]
#![feature(try_trait_v2)]               //https://github.com/rust-lang/rust/issues/84277

mod app;
pub mod computed;
mod css;
mod fetch;
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

// Export log module which can be used in vertigo plugins
pub use log;

pub use app::start_app;

pub use fetch::request_builder::RequestTrait;
pub use fetch::resource::Resource;
pub use fetch::lazy_cache;
pub use websocket::{WebcocketMessageDriver, WebcocketMessage, WebcocketConnection};



#[macro_export]
macro_rules! make_serde_request_trait {
    ($model:ident) => {
        impl RequestTrait for $model {
            fn into_string(self) -> Result<String, String> {
                serde_json::to_string(&self)
                    .map_err(|err| format!("error serialize {}", err))
            }

            fn list_from_string(data: &str) -> Result<Vec<Self>, String> {

                #[derive(Serialize, Deserialize)]
                struct List(Vec<$model>);
        
                let result = serde_json::from_str::<List>(data)
                    .map_err(|err| format!("error deserialize list {}", err))?;
                
                let List(list) = result;
                Ok(list)
            }

            fn from_string(data: &str) -> Result<Self, String> {
                serde_json::from_str::<Self>(data)
                    .map_err(|err| format!("error deserialize {}", err))
            }
        }
    }
}


