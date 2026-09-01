use std::{fs, path::PathBuf};

use crate::{
    build::find_target::{BuildProfile, resolve_target_root, target_dir_opt},
    commons::{ErrorCode, command::CommandRun},
};

use super::Workspace;

const TARGET: &str = "wasm32-unknown-unknown";

/// Drop the package's artifacts so the next build re-expands its macros.
///
/// Uses `cargo clean -p` rather than deleting the artifact by path: the layout of the
/// target directory is not stable across toolchains (stable cargo puts the rlib in
/// `<target>/deps/`, nightly 1.100 in `<target>/build/<pkg>/<hash>/out/`), so a
/// hardcoded path silently stops matching and the bundle ends up missing every asset.
fn run_cargo_clean(
    package_name: &str,
    target_dir_opt: Option<&str>,
    release: bool,
    profile: &BuildProfile,
) -> Result<(), ErrorCode> {
    let mut command = CommandRun::new("cargo")
        .add_param("clean")
        .add_param("--target")
        .add_param(TARGET)
        .add_param("--package")
        .add_param(package_name);

    // Select the same profile and directory the build below will write to. Whatever cargo
    // can work out by itself (CARGO_TARGET_DIR, cargo config) is left to it: passing
    // `--target-dir` explicitly makes cargo apply a CACHEDIR.TAG check that target
    // directories created by older cargo versions fail.
    if profile.from_cargo_opts {
        command = command
            .add_param("--profile")
            .add_param(profile.name.as_str());
    } else if release {
        command = command.add_param("--release");
    }

    if let Some(target_dir) = target_dir_opt {
        command = command.add_param("--target-dir").add_param(target_dir);
    }

    command.set_error_code(ErrorCode::BuildFailed).run()
}

pub fn run_cargo_build(
    package_name: &str,
    vertigo_public_path: &str,
    ws: &Workspace,
    allow_error: bool,
    release: bool,
    cargo_opts: &[String],
) -> Result<Result<PathBuf, String>, ErrorCode> {
    log::info!("Building {package_name}");

    // `--target-dir` and `--profile` can arrive through the caller's own cargo options, and
    // then they - not `cargo metadata`, not `release` - decide where the build lands. Get
    // this wrong and we wipe, clean and read a directory the build never touches.
    let profile = BuildProfile::resolve(release, cargo_opts);
    let target_dir_opt = target_dir_opt(cargo_opts);
    let target_dir = resolve_target_root(ws.get_target_dir(), cargo_opts)
        .join(TARGET)
        .join(&profile.dir);

    // Reset the state that macros accumulate during expansion. Only this command sets
    // VERTIGO_BUNDLE, so only this command can produce it. Note we wipe `static/included`
    // and not `static` itself - the latter also holds wasm_run.js, written by vertigo's
    // build script, which only reruns when the js actually changes.
    let _ = fs::remove_dir_all(target_dir.join("tailwind"));
    let _ = fs::remove_dir_all(target_dir.join("static").join("included"));

    // ...and force the macros to run again to repopulate it. Cargo can't see that proc
    // macros read VERTIGO_BUNDLE and write files, so it would otherwise consider the
    // package fresh and we'd bundle nothing.
    run_cargo_clean(package_name, target_dir_opt, release, &profile)?;

    let mut command = CommandRun::new("cargo").add_param("build");

    if release {
        command = command.add_param("--locked");

        // A `--profile` in cargo_opts already selects the profile, and cargo refuses to
        // take both it and `--release`.
        if !profile.from_cargo_opts {
            command = command.add_param("--release");
        }
    }

    command = command
        .add_param("--target")
        .add_param(TARGET)
        .add_param("--package")
        .add_param(package_name)
        .env("VERTIGO_PUBLIC_PATH", vertigo_public_path)
        // Tell macros that we're bundling so it will produce artifacts
        .env("VERTIGO_BUNDLE", "true")
        // Cargo tells proc macros nothing about the target directory, so hand them the
        // path we got from `cargo metadata`
        .env("VERTIGO_TARGET_DIR", target_dir.to_string_lossy());

    for opt in cargo_opts {
        command = command.add_param(opt);
    }

    if allow_error {
        command = command.allow_error();
    } else {
        command = command.set_error_code(ErrorCode::BuildFailed);
    }

    let (status, output) = command.output_with_status()?;

    if status.success() {
        Ok(Ok(target_dir))
    } else {
        Ok(Err(output))
    }
}
