#![allow(clippy::question_mark)]
use std::path::Path;
use vertigo::dev::VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER;

use crate::commons::{models::IndexModel, ErrorCode};

#[derive(Clone, Debug)]
pub struct MountPathConfig {
    // for http
    mount_point: String,
    // for filesystem
    dest_dir: String,
    // index.json after parsing
    run_js: String,
    // path to wasm-file
    wasm_path: String,
    // whether to preload wasm script using <link rel="preload">
    pub wasm_preload: bool,
    // whether to disable hydration
    pub disable_hydration: bool,
}

impl MountPathConfig {
    pub fn new(
        public_mount_point: String,
        dest_dir: String,
        wasm_preload: bool,
        disable_hydration: bool,
    ) -> Result<MountPathConfig, ErrorCode> {
        let index_model = read_index(&dest_dir)?;

        Ok(MountPathConfig {
            dest_dir,
            mount_point: public_mount_point,
            run_js: index_model.run_js,
            wasm_path: index_model.wasm,
            wasm_preload,
            disable_hydration,
        })
    }

    pub fn mount_point(&self) -> &str {
        self.mount_point.as_str()
    }

    pub fn dest_dir(&self) -> &str {
        self.dest_dir.trim_start_matches("./")
    }

    pub fn dest_http_root(&self) -> String {
        Path::new(&self.mount_point)
            .join(self.dest_dir())
            .components()
            .as_path()
            .to_string_lossy()
            .into_owned()
    }

    pub fn get_wasm_http_path(&self) -> String {
        self.translate_to_http(&self.wasm_path)
    }

    pub fn get_run_js_http_path(&self) -> String {
        self.translate_to_http(&self.run_js)
    }

    pub fn get_wasm_fs_path(&self) -> String {
        self.translate_to_fs(&self.wasm_path)
    }

    fn translate_to_http(&self, fs_path: impl Into<String>) -> String {
        let fs_path = fs_path.into();
        fs_path.replace(
            VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER,
            &self.dest_http_root(),
        )
    }

    fn translate_to_fs(&self, http_path: impl Into<String>) -> String {
        let http_path = http_path.into();
        replace_prefix(&self.dest_dir, &http_path)
    }
}

fn read_index(dest_dir: &str) -> Result<IndexModel, ErrorCode> {
    let index_path = Path::new(dest_dir).join("index.json");
    let index_html = match std::fs::read_to_string(&index_path) {
        Ok(data) => data,
        Err(err) => {
            log::error!("File read error: file={index_path:?}, error={err}, dest_dir={dest_dir}");
            return Err(ErrorCode::ServeCantReadIndexFile);
        }
    };

    serde_json::from_str::<IndexModel>(&index_html).map_err(|err| {
        log::error!("File read error 2: file={index_path:?}, error={err}, dest_dir={dest_dir}");
        ErrorCode::ServeCantReadIndexFile
    })
}

fn replace_prefix(dest_dir: &str, path: &str) -> String {
    if path.starts_with(VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER) {
        // Dynamic path resolution
        path.replace(VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER, dest_dir)
    } else {
        // Static path resolution
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use vertigo::dev::VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER;

    use super::replace_prefix;

    #[test]
    fn test_replace_prefix() {
        assert_eq!(
            replace_prefix("demo_build", "build/vertigo_demo.33.wasm"),
            "build/vertigo_demo.33.wasm".to_string()
        );

        assert_eq!(
            replace_prefix("demo_build", "build/vertigo_demo.33.wasm"),
            "build/vertigo_demo.33.wasm".to_string()
        );

        assert_eq!(
            replace_prefix(
                "demo_build",
                &format!("{VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER}/vertigo_demo.33.wasm")
            ),
            "demo_build/vertigo_demo.33.wasm".to_string()
        );
    }
}
