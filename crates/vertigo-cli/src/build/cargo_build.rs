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

/// Value for `VERTIGO_BUILD_ID`, the variable the bundling macros pull via `option_env!` into
/// the dep-info of each crate they bundle something from - the app and any library with
/// `include_static!` or tailwind classes.
///
/// Cargo can't see that the bundling macros read `VERTIGO_BUNDLE` and write asset files, so
/// left alone it would call those crates fresh and we'd bundle nothing. Handing it a value
/// that differs from last time makes cargo re-expand them - and unlike dropping the
/// crate's artifacts (`cargo clean -p`), the incremental cache survives, which is the
/// difference between a ~1s and a ~7s watch-mode rebuild.
fn build_id() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since_epoch| since_epoch.as_nanos().to_string())
        .unwrap_or_default()
}

/// The variables of our environment that cargo sets for a process it runs, a test binary or
/// `cargo run`: that package's `CARGO_PKG_*` and `CARGO_MANIFEST_*`.
///
/// A cargo started from such a process must go without them. Build scripts track some of them
/// with `rerun-if-env-changed` (ring does), and cargo compares them with its own environment,
/// so that build and the next one from a shell would each find them changed and rebuild those
/// crates, and everything on them.
pub fn inherited_package_env() -> Vec<String> {
    std::env::vars_os()
        .filter_map(|(name, _)| name.into_string().ok())
        .filter(|name| name.starts_with("CARGO_PKG_") || name.starts_with("CARGO_MANIFEST_"))
        .collect()
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

    for name in inherited_package_env() {
        command = command.env_remove(name);
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

#[cfg(test)]
mod tests {
    use super::*;

    // Run by cargo, so with the variables it sets for a test binary
    #[test]
    fn inherited_package_env_is_the_package_identity() {
        let names = inherited_package_env();
        assert!(names.iter().any(|name| name == "CARGO_PKG_NAME"));
        assert!(names.iter().any(|name| name == "CARGO_MANIFEST_DIR"));
        assert!(!names.iter().any(|name| name == "CARGO"));
    }
}
