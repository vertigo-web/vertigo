use std::path::PathBuf;
use serde::Deserialize;
use std::io::ErrorKind;

use super::hash::get_hash;

#[derive(Deserialize, Debug, Clone)]
pub struct EnvConfigIn {
    static_path: String,

    pub public_path: Option<String>,
}
impl EnvConfigIn {
    pub fn get_public(&self) -> String {
        match &self.public_path {
            Some(public_path) => public_path.clone(),
            None => ".".to_string()
        }
    }

    pub fn get_path_to_static(&self, path: impl Into<String>) -> String {
        let path = path.into();
        format!("{}/{}", self.get_public(), path)
    }

    pub fn new_path_in_static_make(&self, path: &[&str]) -> WasmPath {
        let mut result = self.get_static_path();

        for chunk in path {
            result.push(*chunk);
        }

        result
    }

    pub fn new_path_in_static_from(&self, path: &WasmPath) -> WasmPath {
        let name_os = path.file_name();
        self.new_path_in_static_make(&[name_os.as_str()])
    }

    pub fn get_static_path(&self) -> WasmPath {
        WasmPath::new(PathBuf::from(self.static_path.as_str()))
    }
}

#[derive(Debug)]
pub struct WasmPath {
    path: PathBuf,
}

impl WasmPath {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path
        }
    }

    pub fn string(&self) -> String {
        self.path.as_os_str().to_str().unwrap().to_string()
    }

    pub fn file_stem(&self) -> String {
        self.path.file_stem().unwrap().to_str().unwrap().to_string()
    }

    pub fn file_name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
    }

    pub fn file_extension(&self) -> String {
        self.path.extension().unwrap().to_str().unwrap().to_string()
    }

    pub fn save(&self, content: &[u8]) {
        use std::fs::File;
        use std::io::prelude::*;
    
        let mut f = File::create(&self.path).unwrap();
        f.write_all(content).unwrap();
    }

    pub fn save_with_hash(&self, content: &[u8]) -> String {
        let file_name = self.file_stem();
        let hash = get_hash(content);
        let file_ext = self.file_extension();
        
        let parent = self.path.parent().unwrap();
        let target_file_name = format!("{file_name}.{hash}.{file_ext}");
        let target_path = Self::new(parent.join(&target_file_name));

        target_path.save(content);
        target_file_name
    }

    pub fn read(&self) -> Vec<u8> {
        std::fs::read(&self.path).unwrap()
    }

    pub fn push(&mut self, name: impl Into<String>) {
        self.path.push(name.into());
    }

    pub fn remove_dir_all(&self) {
        match std::fs::remove_dir_all(&self.path) {
            Ok(()) => {},
            Err(error) => {
                let kind = error.kind();
                if kind == ErrorKind::NotFound {
                    // ok
                } else {
                    panic!("problem with removing a directory error={error}");
                }
            }
        };
    }

    pub fn create_dir_all(&self) {
        match std::fs::create_dir_all(&self.path) {
            Ok(()) => {},
            Err(error) => {
                println!("directory building error, error={error}");
            }
        }
    }

    pub fn remove_file(&self) {
        std::fs::remove_file(&self.path).unwrap();
    }

    pub fn is_exist(&self) -> bool {
        self.path.exists()
    }
}
