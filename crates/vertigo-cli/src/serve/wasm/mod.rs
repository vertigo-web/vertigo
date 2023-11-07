mod js_value_match;
mod wasm_instance;
mod get_now;
mod data_context;

pub use wasm_instance::{Message, FetchRequest, FetchResponse, WasmInstance};
pub use get_now::get_now;

const VERTIGO_VERSION_MAJOR: u32 = pkg_version::pkg_version_major!();
const VERTIGO_VERSION_MINOR: u32 = pkg_version::pkg_version_minor!();
