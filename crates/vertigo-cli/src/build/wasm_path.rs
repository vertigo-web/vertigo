use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use crc::{Crc, CRC_64_ECMA_182};
use std::{path::PathBuf, io::ErrorKind};

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
        if self.exists() {
            std::fs::remove_file(&self.path).unwrap()
        }
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}

const CRC: Crc<u64> = Crc::<u64>::new(&CRC_64_ECMA_182);

pub fn get_hash(data: &[u8]) -> String {
    let mut hasher = CRC.digest();
    hasher.update(data);
    URL_SAFE_NO_PAD.encode(hasher.finalize().to_be_bytes())
}

#[test]
fn test_hash() {
    let ddd1 = get_hash("vertigo1".as_bytes());
    let ddd2 = get_hash("vertigo1".as_bytes());
    let ddd3 = get_hash("vertigo2".as_bytes());
    assert_eq!(ddd1, ddd2);
    assert_ne!(ddd2, ddd3);
}
