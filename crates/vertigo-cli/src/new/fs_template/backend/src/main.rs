use actix_web::{App, HttpServer, middleware::{Compress, from_fn}};
use vertigo_cli::serve::{vary_accept_encoding, vertigo_install, MountConfigBuilder, ServerState};

const HOST: &str = "127.0.0.1";
const PORT: u16 = 8080;

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    let vertigo_mount_config = MountConfigBuilder::new("/", "./build")
        // Server-side rendering fetches relative URLs (`/api/...`) from this server, as the
        // browser does. Without it they fail during SSR.
        .ssr_fetch_base(format!("http://{HOST}:{PORT}"))
        // A release build of the server serves the wasm and other text files brotli-compressed
        // ahead of time, about a quarter smaller than what `Compress` does per response.
        // `vertigo build` saves them compressed (`name.ext.br`), so the server starts at once.
        .precompress_in_release()
        .build()
        .expect("Failed to create mount config");

    ServerState::init(&vertigo_mount_config).expect("Failed to initialize server state");

    HttpServer::new(move || {
        App::new()
            // The wasm binary and `wasm_run.js` are the bulk of a page load and compress well.
            // `vary_accept_encoding` keeps a cache on the way (nginx, a CDN) from giving every
            // browser what `Compress` sent one client as it is. Drop both lines if a proxy or
            // CDN in front already compresses.
            .wrap(Compress::default())
            .wrap(from_fn(vary_accept_encoding))
            .configure(|srvc| vertigo_install(srvc, &vertigo_mount_config))
    })
    .bind((HOST, PORT))?
    .run()
    .await
}
