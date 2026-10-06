//! Builds what a suite serves: the WASM apps, with the vertigo-cli library, and the backend,
//! with cargo.
//!
//! The apps are built by the same vertigo-cli the backend or [`crate::serve`] uses for SSR, so
//! a vertigo change is always tested on both sides at once.
//!
//! With `E2E_SKIP_BUILD` set, nothing is built, only checked to be there from the last run.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail, ensure};
use vertigo_cli::{
    BuildOpts, CommonOpts,
    build::{self, BuildOptsInner},
};

use crate::env::Settings;

/// Builds the app `package` into `dest_dir`, as `vertigo build` does.
pub async fn vertigo_app(settings: &Settings, package: &str, dest_dir: &Path) -> Result<()> {
    if settings.skip_build {
        println!("E2E_SKIP_BUILD is set, reusing {}", dest_dir.display());
    } else {
        println!("Building {package} into {}", dest_dir.display());

        let opts = BuildOpts {
            inner: BuildOptsInner {
                package_name: Some(package.to_string()),
                public_path: None,
                wasm_opt: Some(settings.wasm_opt),
                release_mode: Some(settings.release),
                wasm_run_source_map: false,
                cargo_opts: vec![],
            },
            common: CommonOpts {
                dest_dir: dest_dir.to_string_lossy().into_owned(),
                log_local_time: None,
            },
        };

        let package = package.to_string();
        tokio::task::spawn_blocking(move || {
            build::run(opts).map_err(|err| anyhow!("building {package} failed: {err:?}"))
        })
        .await??;
    }

    let index = dest_dir.join("index.json");
    ensure!(index.is_file(), "{} is missing", index.display());
    Ok(())
}

/// Builds the binary `name` of the package `name` in the workspace and returns its path.
pub async fn cargo_bin(settings: &Settings, name: &str) -> Result<PathBuf> {
    let bin = settings.target_dir.join(settings.profile()).join(name);

    if !settings.skip_build {
        println!("Building {name}");

        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
        let mut command = tokio::process::Command::new(cargo);
        command.current_dir(&settings.workspace_root).args([
            "build",
            "--package",
            name,
            "--bin",
            name,
        ]);
        if settings.release {
            command.arg("--release");
        }

        let status = command.status().await.context("can't run cargo")?;
        if !status.success() {
            bail!("building {name} failed ({status})");
        }
    }

    ensure!(bin.is_file(), "binary {} is missing", bin.display());
    Ok(bin)
}
