use std::path::PathBuf;

use crate::commons::{command::CommandRun, ErrorCode};

use super::Workspace;

const TARGET: &str = "wasm32-unknown-unknown";
const MODE: &str = "release";

pub fn run_cargo_build(
    package_name: &str,
    vertigo_public_path: &str,
    ws: &Workspace,
    allow_error: bool,
) -> Result<Result<PathBuf, String>, ErrorCode> {
    log::info!("Building {package_name}");

    let mut command = CommandRun::new("cargo")
        .add_param("build")
        .add_param(["--", MODE].concat())
        .add_param("--target")
        .add_param(TARGET)
        .add_param("--package")
        .add_param(package_name)
        .env("VERTIGO_PUBLIC_PATH", vertigo_public_path);

    if allow_error {
        command = command.allow_error();
    } else {
        command = command.set_error_code(ErrorCode::BuildFailed);
    }

    let (status, output) = command.output_with_status()?;

    if status.success() {
        Ok(Ok(ws.get_target_dir().join(TARGET).join(MODE)))
    } else {
        Ok(Err(output))
    }
}
