use actix_web::{
    App, HttpServer,
    middleware::{Compress, Condition},
    rt::System,
};
use std::{num::NonZeroUsize, time::Duration};

use crate::commons::{
    ErrorCode,
    spawn::{ServerOwner, term_signal},
};
use crate::serve::{html::local_origin, mount_path::MountConfig};

use super::{
    ServeOpts, ServeOptsInner, proxy::install_proxy, server_state::ServerState,
    vertigo_install::vertigo_install,
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
        ssr_cookie,
        pretty_html,
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
    mount_config.ssr_cookies = ssr_cookie;
    mount_config.pretty_html = pretty_html;

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
