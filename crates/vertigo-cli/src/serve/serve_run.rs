use actix_web::{
    App, HttpServer,
    middleware::{Compress, Condition, from_fn},
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
    vary::vary_accept_encoding, vertigo_install::vertigo_install,
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
        ssr_fetch_cookie,
        pretty_html,
        disable_compression,
        precompress,
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
    mount_config.ssr_fetch_cookies = ssr_fetch_cookie;
    mount_config.pretty_html = pretty_html;
    mount_config.precompress = precompress;

    ServerState::init_with_watch(&mount_config, port_watch)?;

    let app = move || {
        // Already-encoded responses (what `install_proxy` forwards) carry a `Content-Encoding`
        // and `Compress` leaves those alone.
        let mut app = App::new()
            .wrap(Condition::new(!disable_compression, Compress::default()))
            // Wrapped around `Compress`, it sees what `Compress` did
            .wrap(Condition::new(
                !disable_compression,
                from_fn(vary_accept_encoding),
            ));

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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use actix_web::{App, HttpServer, web};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
    };

    /// The server speaks HTTP/1.1 only. With actix-web's `http2` feature, since actix-web 4.19,
    /// `bind` answers HTTP/2 cleartext (h2c), and actix's HTTP/2 runs on h2 0.3, which has no
    /// fix for RUSTSEC-2026-0258. A client opening with the HTTP/2 preface gets an HTTP/1.1
    /// answer here, where an HTTP/2 server would start with a SETTINGS frame.
    #[actix_web::test]
    async fn a_client_speaking_http2_gets_http1() -> Result<(), Box<dyn std::error::Error>> {
        let server =
            HttpServer::new(|| App::new().route("/", web::get().to(|| async { "Ala ma kota" })))
                .workers(1)
                .bind(("127.0.0.1", 0))?;
        let address = server.addrs()[0];
        let server = server.run();
        let handle = server.handle();
        actix_web::rt::spawn(server);

        let mut stream = TcpStream::connect(address).await?;
        stream
            .write_all(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n")
            .await?;
        let mut answer = vec![0; 64];
        let read = tokio::time::timeout(Duration::from_secs(5), stream.read(&mut answer)).await??;
        handle.stop(false).await;

        let answer = String::from_utf8_lossy(&answer[..read]);
        assert!(answer.starts_with("HTTP/1.1 "), "{answer:?}");
        Ok(())
    }
}
