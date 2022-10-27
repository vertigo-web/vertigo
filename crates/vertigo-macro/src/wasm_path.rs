use sha2::{Sha256, Digest};
use std::{path::PathBuf, /*io::ErrorKind*/};

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

    pub fn as_string(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }

    pub fn file_stem(&self) -> String {
        self.path.file_stem().unwrap().to_string_lossy().into_owned()
    }

    pub fn file_name(&self) -> String {
        self.path.file_name().unwrap().to_string_lossy().into_owned()
    }

    pub fn file_extension(&self) -> String {
        self.path.extension().unwrap().to_string_lossy().into_owned()
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

    pub fn push<P: AsRef<std::path::Path>>(&mut self, names: &[P]) {
        for name in names {
            self.path.push(name);
        }
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}

pub fn get_hash(data: &[u8]) -> String {
    // create a Sha256 object
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();

    hex::encode(&result[..])
}

#[test]
fn test_hash() {
    let ddd = get_hash("vertigo".as_bytes());
    assert_eq!(ddd, "e5a559c8ce04fb73d98cfc83e140713600c1134ac676d0b4debcc9838c09e2d7");
}
