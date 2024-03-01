use std::path::PathBuf;

use crate::commons::command::CommandRun;

use super::Workspace;

const TARGET: &str = "wasm32-unknown-unknown";
const MODE: &str = "release";

pub fn run_cargo_build(
    package_name: &str,
    public_path: &str,
    ws: &Workspace,
) -> Result<PathBuf, String> {
    log::info!("Building {package_name}");

    let (status, output) = CommandRun::new("cargo")
        .add_param("build")
        .add_param(["--", MODE].concat())
        .add_param("--target")
        .add_param(TARGET)
        .add_param("--package")
        .add_param(package_name)
        .env("VERTIGO_PUBLIC_PATH", public_path)
        .error_allowed(true)
        .output_with_status();

    if status.success() {
        Ok(ws.get_target_dir().join(TARGET).join(MODE))
    } else {
        Err(output)
    }
}
