use axum::{
    extract::{State, RawQuery, Json},
    http::{StatusCode, Uri},
    http::{header::HeaderMap},
    response::Response,
    Router, body::BoxBody, routing::{get},
};
use clap::Args;
use serde_json::Value;
use std::{time::{Instant, Duration}, sync::Arc};
use tokio::sync::{OnceCell, RwLock};
use tower_http::services::ServeDir;

use crate::{serve::mount_path::MountPathConfig, utils::parse_key_val};
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

    /// sets up proxy: --proxy /path=http://domain.com/path
    #[arg(long, value_parser = parse_key_val::<String, String>)]
    pub proxy: Vec<(String, String)>,

    /// Setting the parameters --env api=http://domain.com/api --env api2=http://domain.com/api2
    #[arg(long, value_parser = parse_key_val::<String, String>)]
    pub env: Vec<(String, String)>,
}

pub async fn run(opts: ServeOpts, port_watch: Option<u16>) -> Result<(), i32> {
    log::info!("serve params => {opts:#?}");

    let ServeOpts { host, port, dest_dir, proxy, env } = opts;

    let mount_path = MountPathConfig::new(dest_dir)?;
    let state = Arc::new(ServerState::new(mount_path, port_watch, env)?);

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

    let mut app = Router::new()
        .nest_service(&serve_mount_path, serve_dir);

    for (path, target) in proxy {
        app = install_proxy(app, path, target, ref_state.clone());
    }

    let app = app
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

async fn get_response(target_url: String) -> Response<BoxBody> {
    let response = match reqwest::get(target_url.clone()).await {
        Ok(response) => response,
        Err(error) => {
            let message = format!("Error fetching from url={target_url} error={error}");

            let mut response = message.into_response();
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

            return response;
        }
    };

    let headers = response.headers().clone();
    let status = response.status();
    let body = match response.bytes().await {
        Ok(body) => body.to_vec(),
        Err(error) => {
            let message = format!("Error fetching body from url={target_url} error={error}");

            let mut response = message.into_response();
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

            return response;

        }
    };

    use axum::response::IntoResponse;
    let mut response: Response<BoxBody> = body.into_response();

    *response.headers_mut() = headers;
    *response.status_mut() = status;

    response
}

async fn post_response(target_url: String, headers: HeaderMap, body: Value) -> Response<BoxBody> {
    let client = reqwest::Client::new();
    let body = serde_json::to_vec(&body).unwrap();
    let response = match client.post(target_url.clone())
        .headers(headers)
        .body(body).send().await {
        Ok(response) => response,
        Err(error) => {
            let message = format!("Error fetching from url={target_url} error={error}");

            let mut response = message.into_response();
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

            return response;
        }
    };

    let headers = response.headers().clone();
    let status = response.status();
    let body = match response.bytes().await {
        Ok(body) => body.to_vec(),
        Err(error) => {
            let message = format!("Error fetching body from url={target_url} error={error}");

            let mut response = message.into_response();
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

            return response;

        }
    };

    use axum::response::IntoResponse;
    let mut response: Response<BoxBody> = body.into_response();

    *response.headers_mut() = headers;
    *response.status_mut() = status;

    response
}

fn install_proxy(
    app: Router<Arc<RwLock<Arc<ServerState>>>>,
    path: String,
    target: String,
    ref_state: Arc<RwLock<Arc<ServerState>>>
) -> Router<Arc<RwLock<Arc<ServerState>>>> {
    let router = Router::new().fallback(get ({
        let path = path.clone();
        let target = target.clone();

        move |url: Uri| {
            async move {
                let from_url = format!("{path}{url}");
                let target_url = format!("{target}{url}");
                log::info!("proxy get {from_url} -> {target_url}");

                get_response(target_url).await
            }
        }
    }).post({
        let path = path.clone();

        move |url: Uri, headers: HeaderMap, body: Json<Value>| {
            async move {
                let from_url = format!("{path}{url}");
                let target_url = format!("{target}{url}");
                let Json(body) = body;
                log::info!("proxy post {from_url} -> {target_url}");

                post_response(target_url, headers, body).await
            }
        }
    })).with_state(ref_state);

    app.nest_service(path.as_str(), router)
}

#[axum::debug_handler]
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
