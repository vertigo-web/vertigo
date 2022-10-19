use std::path::PathBuf;

use super::config::WasmPath;

pub fn find_wasm_from_target() -> WasmPath {
    let cargo_crate_name = std::env::var("CARGO_CRATE_NAME").unwrap();
    let path = PathBuf::from(format!("target/wasm32-unknown-unknown/release/{cargo_crate_name}.wasm"));
    WasmPath::new(path)
}
