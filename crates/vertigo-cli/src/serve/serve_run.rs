use actix_files::Files;
use actix_proxy::IntoHttpResponse;
use actix_web::{
    dev::{ServiceFactory, ServiceRequest},
    http::StatusCode,
    rt::System,
    web, App, HttpRequest, HttpResponse, HttpServer,
};
use std::{num::NonZeroUsize, time::Instant};

use crate::commons::{
    spawn::{term_signal, ServerOwner},
    ErrorCode,
};
use crate::serve::mount_path::MountPathConfig;

use super::{server_state::ServerState, ServeOpts, ServeOptsInner};

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
        threads,
    } = opts.inner;

    let mount_config = MountPathConfig::new(
        mount_point,
        opts.common.dest_dir,
        wasm_preload,
        disable_hydration,
    )?;

    ServerState::init(mount_config.clone(), port_watch, env)?;

    let serve_mount_path = mount_config.dest_http_root();

    let app = move || {
        let mut app = App::new().service(Files::new(&serve_mount_path, mount_config.dest_dir()));

        for (path, target) in &proxy {
            app = install_proxy(app, path.clone(), target.clone());
        }

        app.default_service(web::to(handler))
    };

    let addr = format!("{host}:{port}");

    let server =
        HttpServer::new(app)
            .workers(threads.unwrap_or_else(|| {
                std::thread::available_parallelism().map_or(2, NonZeroUsize::get)
            }))
            .bind(addr.clone())
            .map_err(|err| {
                log::error!("Can't bind/serve on {addr}: {err}");
                ErrorCode::ServeCantOpenPort
            })?;

    let server = server.disable_signals().run();
    let handle = server.handle();
    let handle2 = server.handle();

    std::thread::spawn(move || System::new().block_on(server));

    tokio::select! {
        _ = ServerOwner { handle } => {},
        msg = term_signal() => {
            log::info!("{msg} received, shutting down");
            handle2.stop(true).await;
        }
    }

    Ok(())
}

fn install_proxy<T>(app: App<T>, path: String, target: String) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = actix_web::Error, InitError = ()>,
{
    app.service(web::scope(&path).default_service(web::to({
        move |req: HttpRequest, body: web::Bytes| {
            let path = path.clone();
            let target = target.clone();
            async move {
                let method = req.method();
                let uri = req.uri();
                let current_path = uri.path();
                let tail = current_path.strip_prefix(&path).unwrap_or(current_path);
                let query = uri.query().map(|q| format!("?{}", q)).unwrap_or_default();

                let target_url = format!("{target}{tail}{query}");

                log::info!("proxy {method} {path}{tail} -> {target_url}");

                let request = awc::Client::new().request_from(&target_url, req.head());

                let response = if !body.is_empty() {
                    request.send_body(body)
                } else {
                    request.send()
                };

                match response.await {
                    Ok(response) => response.into_http_response(),
                    Err(error) => {
                        let message = format!("Error fetching from url={target_url} error={error}");
                        HttpResponse::InternalServerError().body(message)
                    }
                }
            }
        }
    })))
}

async fn handler(req: HttpRequest) -> HttpResponse {
    let state = ServerState::global();

    let now = Instant::now();
    let url = req.uri();

    let uri = {
        let path = url.path();
        // Strip mount point to get local url
        let local_url = if state.mount_config.mount_point() != "/" {
            path.trim_start_matches(state.mount_config.mount_point())
        } else {
            path
        };

        match url.query() {
            Some(query) => format!("{local_url}?{query}"),
            None => local_url.to_string(),
        }
    };

    log::debug!("Incoming request: {uri}");
    let mut response_state = state.request(&uri).await;

    let time = now.elapsed().as_millis();
    let log_level = if time > 1000 {
        log::Level::Warn
    } else {
        log::Level::Info
    };
    log::log!(
        log_level,
        "Response for request: {} {time}ms {uri}",
        response_state.status,
    );

    if let Some(port_watch) = state.port_watch {
        response_state.add_watch_script(port_watch);
    }

    // Checking for error status to log
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
