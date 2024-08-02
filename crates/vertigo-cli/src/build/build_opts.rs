use clap::Args;
use std::path::PathBuf;

use crate::commons::models::CommonOpts;

use super::wasm_path::WasmPath;

#[derive(Args, Debug, Clone)]
pub struct BuildOpts {
    #[clap(flatten)]
    pub common: CommonOpts,
    #[clap(flatten)]
    pub inner: BuildOptsInner,
}

#[derive(Args, Debug, Clone)]
pub struct BuildOptsInner {
    pub package_name: Option<String>,
    #[arg(long, default_value_t = {"/build".to_string()})]
    pub public_path: String,
    #[arg(short, long)]
    pub disable_wasm_opt: bool,
    #[arg(long)]
    pub wasm_run_source_map: bool,
}

impl BuildOpts {
    pub fn public_path_to(&self, path: impl Into<String>) -> String {
        let path = path.into();
        format!("{}/{path}", self.inner.public_path)
    }

    pub fn new_path_in_static_make(&self, path: &[&str]) -> WasmPath {
        let mut result = self.get_dest_dir();

        for chunk in path {
            result.push(*chunk);
        }

        result
    }

    pub fn new_path_in_static_from(&self, path: &WasmPath) -> WasmPath {
        let name_os = path.file_name();
        self.new_path_in_static_make(&[name_os.as_str()])
    }

    pub fn get_dest_dir(&self) -> WasmPath {
        WasmPath::new(PathBuf::from(self.common.dest_dir.as_str()))
    }
}
