use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use crc::{Crc, CRC_64_ECMA_182};
use std::{error::Error, path::PathBuf};

#[derive(Debug)]
pub(crate) struct WasmPath {
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

    pub fn save(&self, content: &[u8]) -> Result<(), Box<dyn Error>> {
        use std::fs::File;
        use std::io::prelude::*;

        let mut f = File::create(&self.path)?;
        f.write_all(content)?;
        Ok(())
    }

    pub fn save_with_hash(&self, content: &[u8]) -> Result<String, Box<dyn Error>> {
        let file_name = self.file_stem()?;
        let hash = get_hash(content);
        let file_ext = self.file_extension()?;

        let parent = self.path.parent().ok_or("Unable to get parent directory")?;
        let target_file_name = format!("{file_name}.{hash}.{file_ext}");
        let target_path = Self::new(parent.join(&target_file_name));

        target_path.save(content)?;
        Ok(target_file_name)
    }

    pub fn read(&self) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(&self.path)
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
