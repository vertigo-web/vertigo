use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn main() {
    // Save PROFILE to env variable for use by bundling macros. Directly used by get_target_dir function.
    export_var("PROFILE");

    probe_default_field_values();

    // No `rerun-if-*` directive here on purpose: a PROFILE change already yields a
    // different build-script unit, and emitting any of them would narrow cargo from
    // tracking every source of this crate down to the listed paths only.
}

fn export_var(name: &str) {
    let value = &env::var(name).unwrap_or_else(|err| {
        panic!("Can't read {name} env variable in vertigo-macro build script: {err}")
    });
    println!("cargo:rustc-env=VERTIGO_{name}={value}");
}

/// Default component props (`#[default = ...]` on a component param, `{..}` in `dom!`) are
/// struct field defaults underneath (rust-lang/rust#132162), so they are turned on only when
/// the compiler building this crate - the same one that builds the crates using the macros -
/// accepts them:
///
/// - `vertigo_default_props` - the macros emit field defaults and `{..}`,
/// - `vertigo_default_props_unstable` - the compiler has them only behind
///   `#![feature(default_field_values)]`, so `#[component]` emits them under
///   `allow_internal_unstable`, which spares the crate declaring a component the feature flag.
///   A crate using `{..}` still needs it: the compiler checks `S { .. }` against the crate's
///   own features, whatever macro wrote it.
fn probe_default_field_values() {
    println!("cargo:rustc-check-cfg=cfg(vertigo_default_props)");
    println!("cargo:rustc-check-cfg=cfg(vertigo_default_props_unstable)");

    const PROBE: &str = "pub struct S { pub a: u8 = 0 }\npub fn f() -> S { S { .. } }\n";

    if compiles("stable", PROBE) {
        println!("cargo:rustc-cfg=vertigo_default_props");
    } else if compiles(
        "unstable",
        &format!("#![feature(default_field_values, allow_internal_unstable)]\n{PROBE}"),
    ) {
        println!("cargo:rustc-cfg=vertigo_default_props");
        println!("cargo:rustc-cfg=vertigo_default_props_unstable");
    }
}

/// Whether the compiler building this crate builds `source` as a library.
fn compiles(name: &str, source: &str) -> bool {
    let (Some(rustc), Some(out_dir)) = (env::var_os("RUSTC"), env::var_os("OUT_DIR")) else {
        return false;
    };
    let dir = PathBuf::from(out_dir).join(format!("probe_{name}"));
    let file = dir.join("probe.rs");
    if fs::create_dir_all(&dir).is_err() || fs::write(&file, source).is_err() {
        return false;
    }

    let mut cmd = Command::new(rustc);
    cmd.args([
        "--edition=2024",
        "--crate-type=lib",
        "--crate-name=vertigo_probe",
        "--emit=metadata",
        "--cap-lints=allow",
    ])
    .arg("--out-dir")
    .arg(&dir)
    .arg(&file)
    .stdout(Stdio::null())
    .stderr(Stdio::null());

    // The flags this crate is built with, f. ex. `-Zcrate-attr=...`. The environment, with
    // `RUSTC_BOOTSTRAP` in it, is passed on as it is for the same reason.
    if let Ok(flags) = env::var("CARGO_ENCODED_RUSTFLAGS") {
        cmd.args(flags.split('\x1f').filter(|flag| !flag.is_empty()));
    }

    let compiled = cmd.status().is_ok_and(|status| status.success());
    // Leave nothing behind in OUT_DIR, it should hold the same files whatever the probe found.
    let _ = fs::remove_dir_all(&dir);
    compiled
}
