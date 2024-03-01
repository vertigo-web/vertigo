use std::path::PathBuf;

use super::wasm_path::WasmPath;

pub fn find_wasm_in_target(package_name: &str) -> WasmPath {
    let wasm_file_name = package_name.replace('-', "_");
    let path = PathBuf::from(format!(
        "target/wasm32-unknown-unknown/release/{wasm_file_name}.wasm"
    ));
    WasmPath::new(path)
}

pub fn find_package_rlib_in_target(package_name: &str) -> WasmPath {
    let wasm_file_name = package_name.replace('-', "_");
    let path = PathBuf::from(format!(
        "target/wasm32-unknown-unknown/release/deps/lib{wasm_file_name}.rlib"
    ));
    WasmPath::new(path)
}
