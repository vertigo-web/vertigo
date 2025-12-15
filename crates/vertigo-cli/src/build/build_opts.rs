use clap::Args;
use std::path::{Path, PathBuf};
use vertigo::dev::VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER;

use crate::commons::models::CommonOpts;

use super::wasm_path::WasmPath;

#[derive(Args, Debug, Clone)]
pub struct BuildOpts {
    #[clap(flatten)]
    pub inner: BuildOptsInner,
    #[clap(flatten)]
    pub common: CommonOpts,
}

#[derive(Args, Debug, Clone)]
pub struct BuildOptsInner {
    pub package_name: Option<String>,
    /// Hard-code public path so the build can be used statically
    #[arg(long)]
    pub public_path: Option<String>,
    /// Whether to perform WASM optimization with wasm-opt command
    ///
    /// This defaults to false (no optimization) for `watch` command
    /// and to true (optimization) for `build` command
    #[arg(short, long)]
    pub wasm_opt: Option<bool>,
    /// Whether to build WASM using release profile
    ///
    /// This defaults to false (debug profile) for `watch` command
    /// and to true (release profile) for `build` command
    #[arg(short, long)]
    pub release_mode: Option<bool>,
    #[arg(long)]
    pub wasm_run_source_map: bool,
    /// Use external tailwind
    ///
    /// This requires `nodejs`, `npm` and `@tailwindcss/cli` installed.
    #[arg(long)]
    pub external_tailwind: bool,
}

impl BuildOpts {
    pub fn get_public_path(&self) -> String {
        // Use predefined public_path or use VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER to keep public path dynamic.
        self.inner
            .public_path
            .clone()
            .unwrap_or(VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER.to_string())
    }

    pub fn public_path_to(&self, path: impl Into<String>) -> String {
        let public_path_str = self.get_public_path();
        let public_path = Path::new(&public_path_str);
        let path_str = path.into();
        let path = Path::new(&path_str);
        public_path.join(path).to_string_lossy().into_owned()
    }

    pub fn new_path_in_static_make(&self, path: &[&str]) -> WasmPath {
        let mut result = self.get_dest_dir();

        for chunk in path {
            result.push(*chunk);
        }

        result
    }

    pub fn new_path_in_static_from(&self, path: &WasmPath) -> WasmPath {
        let Ok(name_os) = path.file_name() else {
            log::error!("Can't get file name from path: {}", path.as_string());
            return self.new_path_in_static_make(&[]);
        };
        self.new_path_in_static_make(&[name_os.as_str()])
    }

    pub fn get_dest_dir(&self) -> WasmPath {
        WasmPath::new(PathBuf::from(self.common.dest_dir.as_str()))
    }
}
