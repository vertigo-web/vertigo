use clap::Args;
use std::path::PathBuf;

use super::wasm_path::WasmPath;

#[derive(Args)]
pub struct BuildOpts {
    pub package_name: Option<String>,
    #[arg(long, default_value_t = {"./build".to_string()})]
    pub dest_dir: String,
    #[arg(long, default_value_t = {"./".to_string()})]
    pub public_path: String,
}

impl BuildOpts {
    pub fn public_path_to(&self, path: impl Into<String>) -> String {
        let path = path.into();
        format!("{}/{path}", self.public_path)
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
        WasmPath::new(PathBuf::from(self.dest_dir.as_str()))
    }
}
