mod data_context;
mod host_state;
mod message;
mod wasm_instance;

pub use host_state::HostState;
pub use message::Message;
pub use wasm_instance::{WasmInstance, build_linker};

const VERTIGO_VERSION_MAJOR: u32 = pkg_version::pkg_version_major!();
const VERTIGO_VERSION_MINOR: u32 = pkg_version::pkg_version_minor!();
