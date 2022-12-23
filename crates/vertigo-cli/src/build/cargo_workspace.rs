use cargo::core::compiler::CrateType;
use cargo::{Config, CargoResult};
use cargo::core::{Workspace, TargetKind, Verbosity};
use std::env::current_dir;

pub fn get_workspace(config_opt: &mut CargoResult::<Config>) -> Result<Workspace<'_>, String> {
    let config = match config_opt {
        CargoResult::Ok(config) => config,
        CargoResult::Err(err) => {
            return Err(format!("Can't load cargo config: {err}"))
        }
    };

    config.shell().set_verbosity(Verbosity::Normal);

    let cwd = match current_dir() {
        Ok(cwd) => cwd,
        Err(err) => {
            return Err(format!("Can't get current working dir: {err}"))
        }
    };

    match Workspace::new(&cwd.join("Cargo.toml"), config) {
        CargoResult::Ok(ws) => Ok(ws),
        CargoResult::Err(err) => {
            Err(format!("Can't load workspace: {err}"))
        }
    }
}

pub fn infer_package_name() -> Result<String, String> {
    let mut cfg = Config::default();
    let ws = get_workspace(&mut cfg)?;
    for member in ws.default_members() {
        if let Some(lib) = member.library() {
            match lib.kind() {
                TargetKind::Lib(lib_types) => {
                    for lib_type in lib_types {
                        match lib_type {
                            CrateType::Cdylib => {
                                return Ok(member.name().to_string())
                            }
                            _ => continue
                        }
                    }
                }
                _ => continue
            }
        }
    }
    Err("Can't find cdylib package in workspace".to_string())
}
