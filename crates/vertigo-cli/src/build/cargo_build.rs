use std::path::PathBuf;

use crate::{
    build::find_target::profile_name,
    commons::{command::CommandRun, ErrorCode},
};

use super::Workspace;

const TARGET: &str = "wasm32-unknown-unknown";

pub fn run_cargo_build(
    package_name: &str,
    vertigo_public_path: &str,
    ws: &Workspace,
    allow_error: bool,
    release: bool,
    external_tailwind: bool,
) -> Result<Result<PathBuf, String>, ErrorCode> {
    log::info!("Building {package_name}");

    let profile = profile_name(release);

    let mut command = CommandRun::new("cargo").add_param("build");

    if release {
        command = command.add_param("--release");
    }

    command = command
        .add_param("--target")
        .add_param(TARGET)
        .add_param("--package")
        .add_param(package_name)
        .env("VERTIGO_PUBLIC_PATH", vertigo_public_path)
        // Tell macros that we're bundling so it will produce artifacts
        .env("VERTIGO_BUNDLE", "true");

    if external_tailwind {
        command = command.env("VERTIGO_EXT_TAILWIND", "true");
    }

    if allow_error {
        command = command.allow_error();
    } else {
        command = command.set_error_code(ErrorCode::BuildFailed);
    }

    let (status, output) = command.output_with_status()?;

    if status.success() {
        Ok(Ok(ws.get_target_dir().join(TARGET).join(profile)))
    } else {
        Ok(Err(output))
    }
}
