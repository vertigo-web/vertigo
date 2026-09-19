use actix_web::{App, HttpServer, middleware::Compress};
use vertigo_cli::serve::{vertigo_install, MountConfigBuilder, ServerState};

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    let vertigo_mount_config = MountConfigBuilder::new("/", "./build")
        .build()
        .expect("Failed to create mount config");

    ServerState::init(&vertigo_mount_config).expect("Failed to initialize server state");

    HttpServer::new(move || {
        App::new()
            // The wasm binary and `wasm_run.js` are the bulk of a page load and compress well.
            // Drop this line if a proxy or CDN in front already compresses.
            .wrap(Compress::default())
            .configure(|srvc| vertigo_install(srvc, &vertigo_mount_config))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
