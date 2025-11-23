use axum::{
    body::BoxBody,
    extract::{Json, RawQuery},
    http::{header::HeaderMap, HeaderValue, StatusCode, Uri},
    response::Response,
    routing::get,
    Router,
};
use reqwest::header;
use serde_json::Value;
use std::time::{Duration, Instant};
use tower_http::services::ServeDir;

use crate::serve::mount_path::MountPathConfig;
use crate::{
    commons::ErrorCode,
    serve::{server_state::ServerState, ServeOptsInner},
};
use axum::body::Body;

use super::ServeOpts;

pub async fn run(opts: ServeOpts, port_watch: Option<u16>) -> Result<(), ErrorCode> {
    log::info!("serve params => {opts:#?}");

    let ServeOptsInner {
        host,
        port,
        proxy,
        mount_point,
        env,
        wasm_preload,
        disable_hydration,
    } = opts.inner;

    let mount_config = MountPathConfig::new(
        mount_point,
        opts.common.dest_dir,
        wasm_preload,
        disable_hydration,
    )?;

    ServerState::init(mount_config.clone(), port_watch, env)?;

    let serve_mount_path = mount_config.dest_http_root();
    let serve_dir = ServeDir::new(mount_config.dest_dir());

    let mut app = Router::new()
        .nest_service(&serve_mount_path, serve_dir)
        .layer(axum::middleware::map_response(set_cache_header));

    for (path, target) in proxy {
        app = install_proxy(app, path, target);
    }

    let app = app.fallback(handler);

    let Ok(addr) = format!("{host}:{port}").parse() else {
        log::error!("Incorrect listening address");
        return Err(ErrorCode::ServeCantOpenPort);
    };

    let ret = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await;

    if let Err(err) = ret {
        log::error!("Can't bind/serve on {addr}: {err}");
        Err(ErrorCode::ServeCantOpenPort)
    } else {
        log::info!("Listening on http://{addr}");
        Ok(())
    }
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

    let Ok(body) = serde_json::to_vec(&body)
        .inspect_err(|err| log::error!("Error serializing request body: {err}"))
    else {
        let mut resp = Response::default();
        *resp.status_mut() = StatusCode::from_u16(600).unwrap_or_default();
        return resp;
    };

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

fn install_proxy(app: Router<()>, path: String, target: String) -> Router<()> {
    let router = Router::new().fallback(
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
    );

    app.nest_service(path.as_str(), router)
}

#[axum::debug_handler]
async fn handler(url: Uri, RawQuery(query): RawQuery) -> Response<Body> {
    let state = ServerState::global();

    let now = Instant::now();
    let uri = {
        // Strip mount point to get local url
        let local_url = if state.mount_config.mount_point() != "/" {
            url.path()
                .trim_start_matches(state.mount_config.mount_point())
        } else {
            url.path()
        };

        match query {
            Some(query) => format!("{local_url}?{query}"),
            None => local_url.to_string(),
        }
    };

    log::debug!("Incoming request: {uri}");
    let mut response_state = state.request(&uri).await;

    let time = now.elapsed();
    log::log!(
        if time > Duration::from_secs(1) {
            log::Level::Warn
        } else {
            log::Level::Info
        },
        "Response for request: {} {}ms {url}",
        response_state.status,
        time.as_millis()
    );

    if let Some(port_watch) = state.port_watch {
        response_state.add_watch_script(port_watch);
    }

    if StatusCode::from_u16(response_state.status)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
        .is_server_error()
    {
        log::error!("WASM status: {}", response_state.status);

        match String::from_utf8(response_state.body.clone()) {
            Ok(messagee) => {
                log::error!("WASM response: text={}", messagee);
            }
            Err(_) => {
                log::error!("WASM response: bytes={:#?}", response_state.body);
            }
        }
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
