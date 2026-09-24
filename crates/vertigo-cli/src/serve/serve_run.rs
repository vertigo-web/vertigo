use actix_proxy::IntoHttpResponse;
use actix_web::{
    App, HttpRequest, HttpResponse, HttpServer,
    dev::{ServiceFactory, ServiceRequest},
    http::header,
    middleware::{Compress, Condition},
    rt::System,
    web,
};
use std::{num::NonZeroUsize, time::Duration};

use crate::commons::{
    ErrorCode,
    spawn::{ServerOwner, term_signal},
};
use crate::serve::{html::local_origin, mount_path::MountConfig};

use super::{
    ServeOpts, ServeOptsInner, server_state::ServerState, vertigo_install::vertigo_install,
};

async fn wait_for_port(addr: &str, port: u16) {
    for i in 0..20 {
        match std::net::TcpListener::bind(addr) {
            Ok(listener) => {
                // Drop the listener to free the port for actix
                drop(listener);
                break;
            }
            Err(_) => {
                log::warn!(
                    "Port {} is still in use, waiting 1s... ({}/20)",
                    port,
                    i + 1
                );
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

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
        ssr_fetch_base,
        disable_compression,
        threads,
    } = opts.inner;

    let mut mount_config = MountConfig::new(
        mount_point,
        opts.common.dest_dir,
        env,
        wasm_preload,
        disable_hydration,
    )?;
    // By default through this server, so relative URLs hit `--proxy` like the browser's requests
    mount_config.ssr_fetch_base = Some(ssr_fetch_base.unwrap_or_else(|| local_origin(&host, port)));

    ServerState::init_with_watch(&mount_config, port_watch)?;

    let app = move || {
        // Already-encoded responses (what `install_proxy` forwards) carry a `Content-Encoding`
        // and `Compress` leaves those alone.
        let mut app = App::new().wrap(Condition::new(!disable_compression, Compress::default()));

        for (path, target) in &proxy {
            app = install_proxy(app, path.clone(), target.clone());
        }

        app.configure(|cfg| {
            vertigo_install(cfg, &mount_config);
        })
    };

    let addr = format!("{host}:{port}");

    wait_for_port(&addr, port).await;

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

    let server = server
        .disable_signals()
        .client_disconnect_timeout(Duration::from_secs(2))
        .shutdown_timeout(5)
        .run();

    let handle = server.handle();
    let handle2 = server.handle();

    std::thread::spawn(move || System::new().block_on(server));

    tokio::select! {
        _ = ServerOwner { handle } => {},
        msg = term_signal() => {
            log::info!("{msg} received, shutting down");
            handle2.stop(false).await;
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
                    Ok(response) => {
                        let mut response = response.into_http_response();

                        let headers = response.headers_mut();

                        // `awc` decompresses the upstream body transparently, but
                        // `into_http_response` copies the upstream headers across verbatim
                        // so remove appropriate headers.
                        if headers.remove(header::CONTENT_ENCODING).next().is_some() {
                            headers.remove(header::CONTENT_LENGTH);
                        }

                        response
                    }
                    Err(error) => {
                        let message = format!("Error fetching from url={target_url} error={error}");
                        HttpResponse::InternalServerError().body(message)
                    }
                }
            }
        }
    })))
}
