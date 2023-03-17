use axum::{
    extract::{State, RawQuery},
    http::{StatusCode, Uri},
    response::Response,
    Router,
};
use clap::Args;
use std::{time::{Instant, Duration}, sync::Arc};
use tokio::sync::{OnceCell, RwLock};
use tower_http::services::ServeDir;

use crate::serve::mount_path::MountPathConfig;
use crate::serve::server_state::ServerState;

static STATE: OnceCell<Arc<RwLock<Arc<ServerState>>>> = OnceCell::const_new();

#[derive(Args, Debug)]
pub struct ServeOpts {
    #[arg(long, default_value_t = {"./build".to_string()})]
    pub dest_dir: String,
    #[arg(long, default_value_t = {"127.0.0.1".into()})]
    pub host: String,
    #[arg(long, default_value_t = {4444})]
    pub port: u16,
    pub port_watch: Option<u16>,
}

pub async fn run(opts: ServeOpts) -> Result<(), i32> {
    let ServeOpts { host, port, port_watch, dest_dir } = opts;
    let mount_path = MountPathConfig::new(dest_dir)?;
    let state = Arc::new(ServerState::new(mount_path, port_watch)?);

    let ref_state = STATE.get_or_init({
        let state = state.clone();

        move || {
            Box::pin(async move {
                Arc::new(RwLock::new(state))
            })
        }
    }).await;

    let serve_mount_path = state.mount_path.http_root();
    let serve_dir = ServeDir::new(
        state.mount_path.fs_root()
    );

    *(ref_state.write().await) = state;

    let app = Router::new()
        .nest_service(&serve_mount_path, serve_dir)
        .fallback(handler)
        .with_state(ref_state.clone());

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


async fn handler(url: Uri, RawQuery(query): RawQuery, State(state): State<Arc<RwLock<Arc<ServerState>>>>) -> Response<String> {
    let state = state.read().await.clone();

    let now = Instant::now();
    let uri = {
        let url = url.path();

        match query {
            Some(query) => format!("{url}?{query}"),
            None => url.to_string(),
        }
    };

    log::debug!("Incoming request: {uri}");
    let (status, mut response) = state.request(&uri).await;

    let time = now.elapsed();
    if time > Duration::from_secs(1) {
        log::warn!("Response for request: {status} {}ms {url}", time.as_millis());
    } else {
        log::info!("Response for request: {status} {}ms {url}", time.as_millis());
    }

    if status == StatusCode::OK {
        if let Some(port_watch) = state.port_watch {
            response = add_watch_script(response, port_watch);
        }
    }

    Response::builder()
        .status(status)
        .header("cache-control", "private, no-cache, no-store, must-revalidate, max-age=0")
        .body(response)
        .unwrap()
}

fn add_watch_script(response: String, port_watch: u16) -> String {
    let watch = include_str!("./watch.js");

    let start = format!("start_watch('http://127.0.0.1:{port_watch}/events');");

    let chunks = vec!(
        "<script>".to_string(),
        watch.to_string(),
        start,
        "</script>".to_string()
    );

    let script = chunks.join("\n");

    format!("{response}\n{script}")
}
