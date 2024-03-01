mod data_context;
mod get_now;
mod js_value_match;
mod wasm_instance;

pub use get_now::get_now;
pub use wasm_instance::{FetchRequest, FetchResponse, Message, WasmInstance};

const VERTIGO_VERSION_MAJOR: u32 = pkg_version::pkg_version_major!();
const VERTIGO_VERSION_MINOR: u32 = pkg_version::pkg_version_minor!();
