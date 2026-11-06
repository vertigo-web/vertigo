use actix_web::{App, HttpServer, middleware::{Compress, from_fn}};
use vertigo_cli::serve::{vary_accept_encoding, vertigo_install, MountConfigBuilder, ServerState};

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    let vertigo_mount_config = MountConfigBuilder::new("/", "./build")
        // A release build of the server compresses the wasm and other text files once, at
        // startup, about a quarter smaller than `Compress` does per response.
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
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
