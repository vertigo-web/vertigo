use actix_files::Files;
use actix_web::web;

use crate::serve::MountConfig;

use super::{
    precompress::precompressed_files, server_state::ServerState, vertigo_handler::vertigo_handler,
};

/// Install vertigo handler and static files based on mount config.
///
/// The handler can work only if ServerState is initialized before.
/// This can't be done automatically because initialization compiles the WASM which takes some time
///
/// Example:
///
/// ```no_run
/// use actix_web::{web, App};
/// use vertigo_cli::serve::{MountConfigBuilder, ServerState, vertigo_install};
///
/// let mount_config = MountConfigBuilder::new("/", "./build").build().unwrap();
/// ServerState::init(&mount_config).unwrap();
///
/// let app = App::new();
///
/// app.configure(|srvc| vertigo_install(srvc, &mount_config));
/// ```
///
/// To bootstrap a project using this method, use fullstack template; run:
///
/// ```sh
/// vertigo-cli new -t fullstack my_project
/// ```
///
pub fn vertigo_install(cfg: &mut web::ServiceConfig, mount_config: &MountConfig) {
    let mount_point = mount_config.mount_point();

    vertigo_files(cfg, mount_config);

    if mount_point == "/" {
        cfg.default_service(vertigo_handler(mount_config));
    } else {
        cfg.service(web::scope(mount_point).default_service(vertigo_handler(mount_config)));
    };
}

/// Install only the build's static files, the precompressed ones too if
/// [`MountConfigBuilder::precompress`](crate::serve::MountConfigBuilder::precompress) asked for
/// them - for a back-end that renders the pages by itself, f. ex. with redirects before SSR,
/// and calls [`vertigo_handler`] or [`ServerState`] on its own.
///
/// [`vertigo_install`] does this and then mounts the handler. Like it, it needs ServerState
/// initialized before.
///
/// ```no_run
/// use actix_web::{web, App, HttpResponse};
/// use vertigo_cli::serve::{MountConfigBuilder, ServerState, vertigo_files};
///
/// let mount_config = MountConfigBuilder::new("/", "./build").precompress(true).build().unwrap();
/// ServerState::init(&mount_config).unwrap();
///
/// let app = App::new()
///     .configure(|cfg| vertigo_files(cfg, &mount_config))
///     .default_service(web::to(|| async { HttpResponse::Ok().body("my own page") }));
/// ```
pub fn vertigo_files(cfg: &mut web::ServiceConfig, mount_config: &MountConfig) {
    if mount_config.precompress {
        let files = ServerState::global(mount_config.mount_point()).precompressed();
        cfg.service(precompressed_files(&mount_config.dest_http_root(), files));
    }

    cfg.service(Files::new(
        &mount_config.dest_http_root(),
        mount_config.dest_dir(),
    ));
}

#[cfg(test)]
mod tests {
    use actix_web::{
        App, HttpResponse,
        http::{StatusCode, header},
        test::{TestRequest, call_service, init_service, read_body},
        web,
    };
    use vertigo::dev::VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER;

    use super::vertigo_files;
    use crate::serve::{MountConfigBuilder, ServerState};

    #[actix_web::test]
    async fn files_go_precompressed_and_pages_to_the_back_end() -> std::io::Result<()> {
        let dest_dir = std::env::temp_dir().join(format!("vertigo-files-{}", std::process::id()));
        std::fs::create_dir_all(&dest_dir)?;
        std::fs::write(
            dest_dir.join("index.json"),
            format!(
                r#"{{"run_js": "{VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER}/wasm_run.js", "wasm": "{VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER}/app.wasm"}}"#
            ),
        )?;
        // An empty module is enough for `ServerState` to start
        std::fs::write(dest_dir.join("app.wasm"), b"\0asm\x01\0\0\0")?;
        let script = "console.log('vertigo');\n".repeat(100);
        std::fs::write(dest_dir.join("wasm_run.js"), &script)?;

        // Its own mount point, as the state is global for the tests running alongside
        let Ok(mount_config) = MountConfigBuilder::new("/files", dest_dir.to_string_lossy())
            .precompress(true)
            .build()
        else {
            panic!("index.json in {dest_dir:?} should read");
        };
        let Ok(()) = ServerState::init(&mount_config) else {
            panic!("the build in {dest_dir:?} should load");
        };

        let app = init_service(
            App::new()
                .configure(|cfg| vertigo_files(cfg, &mount_config))
                .default_service(web::to(|| async { HttpResponse::Ok().body("page") })),
        )
        .await;

        let get = |uri: &str, accept_encoding: &str| {
            TestRequest::get()
                .uri(uri)
                .insert_header((header::ACCEPT_ENCODING, accept_encoding.to_string()))
                .to_request()
        };
        let script_uri = format!("{}/wasm_run.js", mount_config.dest_http_root());

        let response = call_service(&app, get(&script_uri, "gzip, br")).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_ENCODING),
            Some(&header::HeaderValue::from_static("br"))
        );

        let response = call_service(&app, get(&script_uri, "gzip")).await;
        assert_eq!(response.headers().get(header::CONTENT_ENCODING), None);
        assert_eq!(read_body(response).await, script.as_bytes());

        let response = call_service(&app, get("/files/some/page", "br")).await;
        assert_eq!(read_body(response).await, "page".as_bytes());

        std::fs::remove_dir_all(&dest_dir)
    }
}
