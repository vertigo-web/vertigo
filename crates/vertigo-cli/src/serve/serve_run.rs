use axum::{
    body::BoxBody,
    extract::{Json, RawQuery, State},
    http::{header::HeaderMap, HeaderValue, StatusCode, Uri},
    response::Response,
    routing::get,
    Router,
};
use reqwest::header;
use serde_json::Value;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{OnceCell, RwLock};
use tower_http::services::ServeDir;

use crate::serve::{server_state::ServerState, ServeOptsInner};
use crate::serve::mount_path::MountPathConfig;

use super::ServeOpts;

static STATE: OnceCell<Arc<RwLock<Arc<ServerState>>>> = OnceCell::const_new();

pub async fn run(opts: ServeOpts, port_watch: Option<u16>) -> Result<(), i32> {
    log::info!("serve params => {opts:#?}");

    let ServeOptsInner {
        host,
        port,
        proxy,
        env,
    } = opts.inner;

    let mount_path = MountPathConfig::new(opts.common.dest_dir)?;
    let state = Arc::new(ServerState::new(mount_path, port_watch, env)?);

    let ref_state = STATE
        .get_or_init({
            let state = state.clone();

            move || Box::pin(async move { Arc::new(RwLock::new(state)) })
        })
        .await;

    let serve_mount_path = state.mount_path.http_root();

    let serve_dir = ServeDir::new(state.mount_path.fs_root());

    *(ref_state.write().await) = state;

    let mut app = Router::new().nest_service(&serve_mount_path, serve_dir)
        .layer(axum::middleware::map_response(set_cache_header));

    for (path, target) in proxy {
        app = install_proxy(app, path, target, ref_state.clone());
    }

    let app = app.fallback(handler).with_state(ref_state.clone());

    let Ok(addr) = format!("{host}:{port}").parse() else {
        log::error!("Incorrect listening address");
        return Err(-1);
    };

    log::info!("Listening on http://{}", addr);
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
    let response = match client
        .post(target_url.clone())
        .headers(headers)
        .body(body)
        .send()
        .await
    {
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
    ref_state: Arc<RwLock<Arc<ServerState>>>,
) -> Router<Arc<RwLock<Arc<ServerState>>>> {
    let router = Router::new()
        .fallback(
            get({
                let path = path.clone();
                let target = target.clone();

                move |url: Uri| async move {
                    let from_url = format!("{path}{url}");
                    let target_url = format!("{target}{url}");
                    log::info!("proxy get {from_url} -> {target_url}");

                    get_response(target_url).await
                }
            })
            .post({
                let path = path.clone();

                move |url: Uri, headers: HeaderMap, body: Json<Value>| async move {
                    let from_url = format!("{path}{url}");
                    let target_url = format!("{target}{url}");
                    let Json(body) = body;
                    log::info!("proxy post {from_url} -> {target_url}");

                    post_response(target_url, headers, body).await
                }
            }),
        )
        .with_state(ref_state);

    app.nest_service(path.as_str(), router)
}

#[axum::debug_handler]
async fn handler(
    url: Uri,
    RawQuery(query): RawQuery,
    State(state): State<Arc<RwLock<Arc<ServerState>>>>,
) -> Response<String> {
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
    let mut response_state = state.request(&uri).await;

    let time = now.elapsed();
    log::log!(
        if time > Duration::from_secs(1) { log::Level::Warn } else { log::Level::Info },
        "Response for request: {} {}ms {url}",
        response_state.status,
        time.as_millis()
    );

    if let Some(port_watch) = state.port_watch {
        response_state.add_watch_script(port_watch);
    }

    if !response_state.status.is_success() {
        log::error!("WASM status: {}", response_state.status);
        log::error!("WASM response: {}", response_state.body);
    }

    response_state.into()
}

async fn set_cache_header<B: Send>(mut response: Response<B>) -> Response<B> {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000"),
    );
   response
}
