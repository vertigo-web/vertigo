use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use crc::{CRC_64_ECMA_182, Crc};
use std::{error::Error, io::ErrorKind, path::PathBuf};

use crate::commons::ErrorCode;

#[derive(Debug)]
pub struct WasmPath {
    path: PathBuf,
}

impl WasmPath {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn as_string(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }

    pub fn file_stem(&self) -> Result<String, Box<dyn Error>> {
        Ok(self
            .path
            .file_stem()
            .ok_or("Unable to get file stem")?
            .to_string_lossy()
            .into_owned())
    }

    pub fn file_name(&self) -> Result<String, Box<dyn Error>> {
        Ok(self
            .path
            .file_name()
            .ok_or("Unable to get file name")?
            .to_string_lossy()
            .into_owned())
    }

    pub fn file_extension(&self) -> Result<String, Box<dyn Error>> {
        Ok(self
            .path
            .extension()
            .ok_or("Unable to get file extension")?
            .to_string_lossy()
            .into_owned())
    }

    fn inner_save(&self, content: &[u8]) -> Result<(), Box<dyn Error>> {
        use std::fs::File;
        use std::io::prelude::*;

        let mut f = File::create(&self.path)?;
        f.write_all(content)?;
        Ok(())
    }

    pub fn save(&self, content: &[u8]) -> Result<(), ErrorCode> {
        self.inner_save(content).map_err(|err| {
            log::error!("Can't write file: {err}");
            ErrorCode::CantWriteOrRemoveFile
        })
    }

    pub fn save_with_hash(&self, content: &[u8]) -> Result<String, ErrorCode> {
        let func = || {
            let file_name = self.file_stem()?;
            let hash = get_hash(content);
            let file_ext = self.file_extension()?;

            let parent = self.path.parent().ok_or("Unable to get parent directory")?;
            let target_file_name = format!("{file_name}.{hash}.{file_ext}");
            let target_path = Self::new(parent.join(&target_file_name));

            target_path.inner_save(content)?;
            Ok(target_file_name)
        };

        func().map_err(|err: Box<dyn Error>| {
            log::error!("Can't write file with hash: {err}");
            ErrorCode::CantWriteOrRemoveFile
        })
    }

    pub fn read(&self) -> Result<Vec<u8>, ErrorCode> {
        std::fs::read(&self.path).map_err(|err| {
            log::error!("Can't read file {} : {err}", self.path.display());
            ErrorCode::CantReadStaticFile
        })
    }

    pub fn push(&mut self, name: impl Into<String>) {
        self.path.push(name.into());
    }

    pub fn remove_dir_all(&self) {
        match std::fs::remove_dir_all(&self.path) {
            Ok(()) => {}
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
            Ok(()) => {}
            Err(error) => {
                println!("directory building error, error={error}");
            }
        }
    }

    pub fn remove_file(&self) -> Result<(), ErrorCode> {
        if self.exists() {
            std::fs::remove_file(&self.path).map_err(|err| {
                log::error!("Can't remove file: {err}");
                ErrorCode::CantWriteOrRemoveFile
            })
        } else {
            Ok(())
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
