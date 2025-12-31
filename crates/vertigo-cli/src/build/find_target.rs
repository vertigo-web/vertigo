use std::path::PathBuf;

use super::wasm_path::WasmPath;

pub fn profile_name(release: bool) -> &'static str {
    if release { "release" } else { "debug" }
}

pub fn get_target_dir(profile: &str) -> PathBuf {
    PathBuf::from(format!("target/wasm32-unknown-unknown/{profile}"))
}

pub fn find_wasm_in_target(package_name: &str, profile: &str) -> WasmPath {
    let base_path = get_target_dir(profile);
    let wasm_file_name = package_name.replace('-', "_");
    let path = base_path.join(format!("{wasm_file_name}.wasm"));
    WasmPath::new(path)
}

pub fn find_package_rlib_in_target(package_name: &str, profile: &str) -> WasmPath {
    let base_path = get_target_dir(profile);
    let wasm_file_name = package_name.replace('-', "_");
    let path = base_path.join(format!("deps/lib{wasm_file_name}.rlib"));
    WasmPath::new(path)
}
