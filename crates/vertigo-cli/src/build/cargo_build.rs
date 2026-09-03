use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    build::find_target::{BuildProfile, resolve_target_root},
    commons::{ErrorCode, command::CommandRun},
};

use super::Workspace;

const TARGET: &str = "wasm32-unknown-unknown";

/// Value for `VERTIGO_BUILD_ID`, the variable `#[vertigo::main]` pulls into the app crate's
/// dep-info via `option_env!`.
///
/// Cargo can't see that the bundling macros read `VERTIGO_BUNDLE` and write asset files, so
/// left alone it would call the app crate fresh and we'd bundle nothing. Handing it a value
/// that differs from last time makes cargo re-expand the crate - and unlike dropping the
/// crate's artifacts (`cargo clean -p`), the incremental cache survives, which is the
/// difference between a ~1s and a ~7s watch-mode rebuild.
fn build_id() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since_epoch| since_epoch.as_nanos().to_string())
        .unwrap_or_default()
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
    let target_dir = resolve_target_root(ws.get_target_dir(), cargo_opts)
        .join(TARGET)
        .join(&profile.dir);

    // Reset the state that macros accumulate during expansion. Only this command sets
    // VERTIGO_BUNDLE, so only this command can produce it. Note we wipe `static/included`
    // and not `static` itself - the latter also holds wasm_run.js, written by vertigo's
    // build script, which only reruns when the js actually changes.
    let _ = fs::remove_dir_all(target_dir.join("tailwind"));
    let _ = fs::remove_dir_all(target_dir.join("static").join("included"));

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
        .env("VERTIGO_TARGET_DIR", target_dir.to_string_lossy())
        // ...and force the macros to run again, so they repopulate what we just wiped
        .env("VERTIGO_BUILD_ID", build_id());

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
