use std::time::{Instant, Duration};

use clap::Args;
use crate::serve::mount_path::MountPathConfig;
use crate::serve::server_state::ServerState;

use axum::{response::Html, Router, http::{Uri, StatusCode}, extract::{State}};
use axum_extra::routing::SpaRouter;

#[derive(Args, Debug)]
pub struct ServeOpts {
    #[arg(long, default_value_t = {"./build".to_string()})]
    pub dest_dir: String,
    #[arg(long, default_value_t = {"127.0.0.1".into()})]
    pub host: String,
    #[arg(long, default_value_t = {4444})]
    pub port: u32,
}

pub async fn run(opts: ServeOpts) -> Result<(), i32> {
    let ServeOpts { host, port, dest_dir } = opts;
    let mount_path = MountPathConfig::new(dest_dir)?;
    let state = ServerState::new(mount_path)?;

    let spa = SpaRouter::new(
        state.mount_path.http_root().as_str(),
        state.mount_path.fs_root()
    );

    async fn handler(uri: Uri, State(state): State<ServerState>) -> (StatusCode, Html<String>) {
        let now = Instant::now();
        let path = uri.path();
        log::debug!("Incoming request: {path}");
        let (status, response) = state.request(path).await;

        let time = now.elapsed();
        if time > Duration::from_secs(1) {
            log::warn!("Response for request: {status} {}ms {path}", time.as_millis());
        } else {
            log::info!("Response for request: {status} {}ms {path}", time.as_millis());
        }

        (status, response)
    }

    let app = Router::new()
        .merge(spa)
        .fallback(handler)
        .with_state(state);

    let Ok(addr) = format!("{host}:{port}").parse() else {
        log::error!("Incorrect listening address");
        return Err(-1);
    };

    log::info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
