mod data_context;
mod decode_commands;
mod get_now;
mod js_value_match;
mod message;
mod wasm_instance;

pub use message::{FetchRequest, FetchResponse, Message};
pub use wasm_instance::WasmInstance;

const VERTIGO_VERSION_MAJOR: u32 = pkg_version::pkg_version_major!();
const VERTIGO_VERSION_MINOR: u32 = pkg_version::pkg_version_minor!();
