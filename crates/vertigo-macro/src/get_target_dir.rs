use std::path::PathBuf;

use crate::{utils::build_profile, wasm_path::WasmPath};

/// Set by vertigo-cli to the target directory reported by `cargo metadata`.
const VERTIGO_TARGET_DIR: &str = "VERTIGO_TARGET_DIR";

/// Directory the bundling macros write their artifacts into.
///
/// Cargo tells build scripts where the target directory is (`OUT_DIR`), but tells proc
/// macros nothing, so we can't derive it here. Instead vertigo-cli - which asks cargo
/// for the authoritative path via `cargo metadata` - passes it down in
/// `VERTIGO_TARGET_DIR`, next to the `VERTIGO_BUNDLE` flag that gates all writing.
///
/// The fallback only guesses, and only matters when something other than vertigo-cli
/// sets `VERTIGO_BUNDLE`: it is relative to rustc's cwd and hardcodes both the triple
/// and the stock profile names, so it disagrees with the real target directory under
/// `CARGO_TARGET_DIR`, `--target-dir`, a custom profile, or a non-wasm target.
pub fn get_target_dir() -> PathBuf {
    if let Ok(dir) = std::env::var(VERTIGO_TARGET_DIR) {
        return PathBuf::from(dir);
    }

    PathBuf::from(format!("target/wasm32-unknown-unknown/{}", build_profile()))
}

pub fn get_target_wasm_dir() -> WasmPath {
    WasmPath::new(get_target_dir())
}

pub fn get_target_wasm_dir_with<P: AsRef<std::path::Path>>(sub_path: &[P]) -> WasmPath {
    let mut path = get_target_wasm_dir();
    path.push(sub_path);
    path
}
