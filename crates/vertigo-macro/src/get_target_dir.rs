use std::path::PathBuf;

use crate::{utils::build_profile, wasm_path::WasmPath};

pub fn get_target_dir_str() -> String {
    // Can't use any dynamic variables here, as proc-macro package is built for host machine type
    // even if the the final outcome is built for wasm32.
    format!("target/wasm32-unknown-unknown/{}", build_profile())
}

pub fn get_target_dir() -> PathBuf {
    PathBuf::from(get_target_dir_str())
}

pub fn get_target_wasm_dir() -> WasmPath {
    WasmPath::new(get_target_dir())
}

pub fn get_target_wasm_dir_with<P: AsRef<std::path::Path>>(sub_path: &[P]) -> WasmPath {
    let mut path = get_target_wasm_dir();
    path.push(sub_path);
    path
}
