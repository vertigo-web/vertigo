//! A build served in this process, as `vertigo serve` does it (SSR included), for apps that
//! need no backend of their own.

use std::path::Path;

use actix_web::{App, HttpServer, dev::ServerHandle, rt::System};
use anyhow::{Context, Result, anyhow};
use vertigo_cli::serve::{MountConfigBuilder, ServerState, vertigo_install};

pub struct Server {
    /// `http://127.0.0.1:<port>`, without a trailing slash.
    pub base_url: String,
    handle: ServerHandle,
}

impl Server {
    /// Serves `build_dir` at `/` on a free port. The directory is given by its absolute path,
    /// so the static files are served under it.
    pub async fn start(build_dir: &Path) -> Result<Self> {
        let mount_config = MountConfigBuilder::new("/", build_dir.to_string_lossy())
            .build()
            .map_err(|err| anyhow!("can't read the build in {}: {err:?}", build_dir.display()))?;

        // Compiles the wasm module for SSR, which takes a while for a debug build
        {
            let mount_config = mount_config.clone();
            tokio::task::spawn_blocking(move || ServerState::init(&mount_config))
                .await?
                .map_err(|err| anyhow!("can't load {} for SSR: {err:?}", build_dir.display()))?;
        }

        let server = HttpServer::new(move || {
            App::new().configure(|cfg| vertigo_install(cfg, &mount_config))
        })
        .workers(2)
        .disable_signals()
        .bind(("127.0.0.1", 0))
        .context("can't open a port for the server")?;
        let port = server
            .addrs()
            .first()
            .context("the server has no address")?
            .port();

        let server = server.run();
        let handle = server.handle();
        // Like in `vertigo serve`, actix gets a thread with its own system
        std::thread::spawn(move || System::new().block_on(server));

        Ok(Self {
            base_url: format!("http://127.0.0.1:{port}"),
            handle,
        })
    }

    pub async fn stop(&self) {
        self.handle.stop(false).await;
    }
}
