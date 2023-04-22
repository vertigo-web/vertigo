use cargo::{Config};
use std::path::PathBuf;

use crate::command::CommandRun;

use super::cargo_workspace::get_workspace;

const TARGET: &str = "wasm32-unknown-unknown";
const MODE: &str = "release";

pub fn run_cargo_build(package_name: &str, public_path: &str) -> Result<PathBuf, String> {
    log::info!("Building {package_name}");

    CommandRun::new("cargo")
        .add_param("build")
        .add_param("--release")
        .add_param("--target")
        .add_param("wasm32-unknown-unknown")
        .add_param("--package")
        .add_param(package_name)
        .env("VERTIGO_PUBLIC_PATH", public_path)
        .run();

    let mut config_opt = Config::default();
    let workspace = match get_workspace(&mut config_opt) {
        Ok(ws) => ws,
        Err(err) => {
            let msg = format!("Build failed: {err}");
            log::error!("{}", &msg);
            return Err(msg);
        }
    };

    Ok(workspace.target_dir().join(TARGET).join(MODE).into_path_unlocked())
}
