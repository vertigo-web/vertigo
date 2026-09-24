mod element;
mod element_children;
mod fetch_url;
mod html_element;
mod html_node;
mod html_node_convert_to_string;
mod html_response;
mod send_request;

pub use fetch_url::{SSR_FETCH_HEADER, local_origin};
pub use html_node::HtmlNode;
pub use html_response::HtmlResponse;

mod html_build_response;

mod fetch_cache;
pub use fetch_cache::FetchCache;
