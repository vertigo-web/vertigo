#![allow(clippy::question_mark)]
use std::{collections::HashMap, path::Path, sync::Arc};
use vertigo::dev::{
    VERTIGO_MOUNT_POINT_ENV, VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER, VERTIGO_PUBLIC_PATH_ENV,
};

use crate::commons::{ErrorCode, models::IndexModel};

pub struct MountConfigBuilder {
    pub mount_point: String,
    pub dest_dir: String,
    pub env: Vec<(String, String)>,
    pub wasm_preload: bool,
    pub disable_hydration: bool,
    pub ssr_fetch_base: Option<String>,
}

impl MountConfigBuilder {
    pub fn new(mount_point: impl Into<String>, dest_dir: impl Into<String>) -> MountConfigBuilder {
        MountConfigBuilder {
            mount_point: mount_point.into(),
            dest_dir: dest_dir.into(),
            env: vec![],
            wasm_preload: false,
            disable_hydration: false,
            ssr_fetch_base: None,
        }
    }

    pub fn envs(mut self, envs: Vec<(String, String)>) -> Self {
        self.env = envs;
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    pub fn wasm_preload(mut self, wasm_preload: bool) -> Self {
        self.wasm_preload = wasm_preload;
        self
    }

    pub fn disable_hydration(mut self, disable_hydration: bool) -> Self {
        self.disable_hydration = disable_hydration;
        self
    }

    /// Origin against which SSR resolves relative fetch URLs (`/api/posts`), e.g. `http://127.0.0.1:8080`.
    ///
    /// Without it, fetching a relative URL during SSR fails.
    pub fn ssr_fetch_base(mut self, origin: impl Into<String>) -> Self {
        self.ssr_fetch_base = Some(origin.into());
        self
    }

    pub fn build(self) -> Result<MountConfig, ErrorCode> {
        let mut config = MountConfig::new(
            self.mount_point,
            self.dest_dir,
            self.env,
            self.wasm_preload,
            self.disable_hydration,
        )?;
        config.ssr_fetch_base = self.ssr_fetch_base;
        Ok(config)
    }
}

#[derive(Clone, Debug)]
pub struct MountConfig {
    /// Mount point in URL
    mount_point: String,
    /// Build destination directory (where wasm file and wasm_run.js are stored)
    dest_dir: String,
    /// waasm_run.js taken from index.json
    run_js: String,
    /// path to wasm-file taken from index.json
    wasm_path: String,
    /// Environment variables passed to WASM runtime - together with the mount point and the
    /// public path of the build (`vertigo-mount-point`, `vertigo-public-path`)
    pub env: Arc<HashMap<String, String>>,
    /// Whether to preload wasm script using <link rel="preload">
    pub wasm_preload: bool,
    /// Whether to disable hydration
    pub disable_hydration: bool,
    /// See [`MountConfigBuilder::ssr_fetch_base`]
    pub ssr_fetch_base: Option<String>,
}

impl MountConfig {
    pub fn new(
        public_mount_point: impl Into<String>,
        dest_dir: impl Into<String>,
        env: Vec<(String, String)>,
        wasm_preload: bool,
        disable_hydration: bool,
    ) -> Result<MountConfig, ErrorCode> {
        let dest_dir = dest_dir.into();
        let index_model = read_index(&dest_dir)?;

        let mut config = MountConfig {
            dest_dir,
            mount_point: public_mount_point.into(),
            run_js: index_model.run_js,
            wasm_path: index_model.wasm,
            env: Arc::default(),
            wasm_preload,
            disable_hydration,
            ssr_fetch_base: None,
        };

        // The app gets them wherever it runs - in the browser as `data-env-*`, during SSR from
        // the host - so both sides build the same paths (`Driver::route_to_public`,
        // `Driver::public_build_path`)
        let mut env: HashMap<String, String> = env.into_iter().collect();
        env.insert(VERTIGO_MOUNT_POINT_ENV.into(), config.mount_point.clone());
        env.insert(VERTIGO_PUBLIC_PATH_ENV.into(), config.dest_http_root());
        config.env = Arc::new(env);

        Ok(config)
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

    use super::{MountConfig, replace_prefix};

    #[test]
    fn env_carries_mount_point_and_public_path() -> std::io::Result<()> {
        let dest_dir = std::env::temp_dir().join(format!("vertigo-mount-{}", std::process::id()));
        std::fs::create_dir_all(&dest_dir)?;
        std::fs::write(
            dest_dir.join("index.json"),
            r#"{"run_js": "wasm_run.js", "wasm": "app.wasm"}"#,
        )?;

        let Ok(config) = MountConfig::new(
            "/panel",
            dest_dir.to_string_lossy(),
            vec![
                ("api_url".into(), "/api".into()),
                // the server knows better where it mounted the app
                ("vertigo-mount-point".into(), "/elsewhere".into()),
            ],
            false,
            false,
        ) else {
            panic!("index.json in {dest_dir:?} should read");
        };

        assert_eq!(config.env["api_url"], "/api");
        assert_eq!(config.env["vertigo-mount-point"], "/panel");
        assert_eq!(config.env["vertigo-public-path"], config.dest_http_root());

        std::fs::remove_dir_all(&dest_dir)
    }

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
