use std::fs;
use std::env;
use std::path::PathBuf;

fn main() {
    let dir = PathBuf::from(env::var("OUT_DIR").unwrap())
        .join("../../../static");

    // Invokes build script again if these changed:
    println!("cargo:rerun-if-changed=src/driver_module/wasm_run.js");
    println!("cargo:rerun-if-changed={}", dir.to_string_lossy());

    fs::create_dir_all(&dir).unwrap();

    let error = fs::remove_dir_all(dir.join("included"));
    println!("remove_dir_all => {error:?}");

    // Subdirectory for files included in dom macro invocations
    fs::create_dir_all(dir.join("included")).unwrap();

    let wasm_run_path = dir.join("wasm_run.js");
    let wasm_run_content = include_str!("src/driver_module/wasm_run.js");

    fs::write(&wasm_run_path, wasm_run_content.as_bytes())
        .unwrap_or_else(|_| panic!("Couldn't write to {}!", wasm_run_path.to_string_lossy()));

    println!("Saved {}", wasm_run_path.to_string_lossy());
}
