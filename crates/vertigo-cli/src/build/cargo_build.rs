use cargo::core::compiler::{CompileTarget, CompileKind, MessageFormat};
use cargo::{ops, Config};
use cargo::util::command_prelude::CompileMode;
use std::path::PathBuf;

use crate::logs::{log_ok, log_error};
use super::cargo_workspace::get_workspace;

const TARGET: &str = "wasm32-unknown-unknown";
const MODE: &str = "release";

pub fn run_cargo_build(package_name: &str) -> Result<PathBuf, String> {
    log_ok(format!("Building {package_name}"));

    let mut config_opt = Config::default();
    let workspace = match get_workspace(&mut config_opt) {
        Ok(ws) => ws,
        Err(err) => {
            let msg = format!("Build failed: {}", err);
            log_error(&msg);
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
            log_ok("WASM built successfully");
            Ok(workspace.target_dir().join(TARGET).join(MODE).into_path_unlocked())
        },
        Err(err) => {
            log_error(format!("WASM build failed: {}", err));
            Err(err.to_string())
        }
    }
}
