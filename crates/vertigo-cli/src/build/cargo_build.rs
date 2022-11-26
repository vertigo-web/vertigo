use cargo::core::compiler::{CompileTarget, CompileKind, MessageFormat};
use cargo::{ops, Config};
use cargo::util::command_prelude::CompileMode;
use std::path::PathBuf;

use super::cargo_workspace::get_workspace;

const TARGET: &str = "wasm32-unknown-unknown";
const MODE: &str = "release";

pub fn run_cargo_build(package_name: &str, public_path: &str) -> Result<PathBuf, String> {
    log::info!("Building {package_name}");

    std::env::set_var("VERTIGO_PUBLIC_PATH", public_path);

    let mut config_opt = Config::default();
    let workspace = match get_workspace(&mut config_opt) {
        Ok(ws) => ws,
        Err(err) => {
            let msg = format!("Build failed: {}", err);
            log::error!("{}", &msg);
            return Err(msg);
        }
    };

    let mut options = ops::CompileOptions::new(workspace.config(), CompileMode::Build).unwrap();
    options.spec = ops::Packages::from_flags(false, vec![], vec![package_name.to_string()]).unwrap();
    let target = CompileTarget::new(TARGET).unwrap();
    options.build_config.requested_kinds = vec![CompileKind::Target(target)];
    options.build_config.requested_profile = MODE.into();
    options.build_config.message_format = MessageFormat::Human;
    options.build_config.keep_going = true;

    match ops::compile(&workspace, &options) {
        Ok(_success) => {
            log::info!("WASM built successfully");
            Ok(workspace.target_dir().join(TARGET).join(MODE).into_path_unlocked())
        },
        Err(err) => {
            log::error!("WASM build failed: {err}");
            Err(err.to_string())
        }
    }
}
