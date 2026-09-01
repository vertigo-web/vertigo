use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let target_dir = find_target_dir()?;

    let dir = target_dir.join("static");

    fs::create_dir_all(&dir)?;

    // NOTE: Never declare `cargo:rerun-if-changed` for `dir` or anything below it.
    // It is pure output of this script, and cargo watches directories recursively,
    // so watching it guarantees a false invalidation on every build: `include_static!`
    // writes into `static/included` while downstream crates are being expanded, and
    // every vertigo build-script unit sharing this target dir rewrites these files.
    // Cleanup of `static/included` and `tailwind` belongs to vertigo-cli, which is
    // the only thing that sets VERTIGO_BUNDLE (see cli's cargo_build.rs).

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

fn find_target_dir() -> Result<PathBuf, Box<dyn Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    let target_dir = out_dir
        .ancestors()
        .find(|dir| dir.file_name() == Some(OsStr::new("build")))
        .and_then(Path::parent)
        .ok_or_else(|| format!("Can't find target dir in OUT_DIR: {}", out_dir.display()))?;

    Ok(target_dir.to_path_buf())
}

fn bundle_file(
    in_path: &str,
    content: &str,
    out_dir: &Path,
    file_name: &str,
) -> Result<(), Box<dyn Error>> {
    // Invokes build script again if this file changed. These are the only inputs of
    // this script, and emitting them switches cargo from "watch all sources" to
    // "watch only these paths" (changes to build.rs itself are still covered, via the
    // build script binary's own fingerprint).
    println!("cargo:rerun-if-changed={in_path}");

    let out_path = out_dir.join(file_name);

    fs::write(&out_path, content.as_bytes())?;

    println!("Bundled {}", out_path.to_string_lossy());

    Ok(())
}
