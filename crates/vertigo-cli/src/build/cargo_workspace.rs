use cargo::core::compiler::CrateType;
use cargo::{Config, CargoResult};
use cargo::core::{Workspace, TargetKind, Verbosity};
use std::env::current_dir;

use crate::logs::{log_ok, log_error};

pub fn get_workspace(config_opt: &mut CargoResult::<Config>) -> Option<Workspace<'_>> {
    let config = match config_opt {
        CargoResult::Ok(config) => config,
        CargoResult::Err(err) => {
            log_error(format!("Can't load cargo config: {}", err));
            return None
        }
    };

    config.shell().set_verbosity(Verbosity::Normal);

    let cwd = match current_dir() {
        Ok(cwd) => cwd,
        Err(err) => {
            log_error(&format!("Can't get current working dir: {}", err));
            return None
        }
    };

    match Workspace::new(&cwd.join("Cargo.toml"), config) {
        CargoResult::Ok(ws) => Some(ws),
        CargoResult::Err(err) => {
            log_error(format!("Can't infer package name: {}", err));
            None
        }
    }
}

pub fn infer_package_name() -> Option<String> {
    get_workspace(&mut Config::default()).and_then(|ws| {
        for member in ws.default_members() {
            if let Some(lib) = member.library() {
                match lib.kind() {
                    TargetKind::Lib(lib_types) => {
                        for lib_type in lib_types {
                            match lib_type {
                                CrateType::Cdylib => {
                                    log_ok(format!("Inferred package name = {}", member.name()));
                                    return Some(member.name().to_string())
                                }
                                _ => continue
                            }
                        }
                    }
                    _ => continue
                }
            }
        }
        None
    })
}
