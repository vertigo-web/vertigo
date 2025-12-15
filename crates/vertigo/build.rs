use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let target_dir = PathBuf::from(env::var("OUT_DIR")?).join("../../..");

    let _ = fs::remove_dir_all(target_dir.join("tailwind"));

    let dir = target_dir.join("static");

    fs::create_dir_all(&dir)?;

    if let Err(error) = fs::remove_dir_all(dir.join("included")) {
        eprintln!("remove_dir_all => {error:?}");
    }

    // Subdirectory for files included in dom macro invocations
    fs::create_dir_all(dir.join("included"))?;

    // Invokes build script again if these changed:
    println!("cargo:rerun-if-changed={}", dir.to_string_lossy());

    println!("cargo:rerun-if-changed=src/driver_module/wasm_run.js");
    println!("cargo:rerun-if-changed=src/driver_module/wasm_run.js.map");

    bundle_file(
        "src/driver_module/wasm_run.js",
        include_str!("src/driver_module/wasm_run.js"),
        &dir,
        "wasm_run.js",
    )?;

    bundle_file(
        "src/driver_module/wasm_run.js.map",
        include_str!("src/driver_module/wasm_run.js.map"),
        &dir,
        "wasm_run.js.map",
    )?;

    Ok(())
}

fn bundle_file(
    in_path: &str,
    content: &str,
    out_dir: &Path,
    file_name: &str,
) -> Result<(), Box<dyn Error>> {
    // Invokes build script again if this file changed
    println!("cargo:rerun-if-changed={in_path}");

    let out_path = out_dir.join(file_name);

    fs::write(&out_path, content.as_bytes())?;

    println!("Bundled {}", out_path.to_string_lossy());

    Ok(())
}
