use std::path::PathBuf;

use actix_files::Files;
use actix_web::{dev::Service, web};

use crate::serve::MountConfig;

use super::{
    immutable::{is_build_file_named_by_content, mark_immutable},
    precompress::precompressed_files,
    server_state::ServerState,
    vertigo_handler::vertigo_handler,
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
/// and calls [`vertigo_handler`] or [`ServerState`] on its own. A page handler using
/// [`ServerState`] should refuse the app's SSR fetches, see [`is_ssr_fetch`](crate::serve::is_ssr_fetch).
///
/// [`vertigo_install`] does this and then mounts the handler. Like it, it needs ServerState
/// initialized before.
///
/// A file named by its content (`name.HASH.ext`, as `vertigo build` and `include_static!` save
/// them) goes with `Cache-Control: public, max-age=31536000, immutable`: a new content gets
/// a new name, so a browser and a CDN keep the file without asking again. The rest of the build
/// (`index.json`) the browser checks with its ETag.
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
    let files = web::scope(&mount_config.dest_http_root());
    let files = match mount_config.precompress {
        true => {
            let precompressed = ServerState::global(mount_config.mount_point()).precompressed();
            // in the scope, so its paths start after the build's root
            files.service(precompressed_files("", precompressed))
        }
        false => files,
    };

    let dest_dir = PathBuf::from(mount_config.dest_dir());
    cfg.service(
        files
            .service(Files::new("", mount_config.dest_dir()))
            .wrap_fn(move |req, service| {
                let named = is_build_file_named_by_content(&dest_dir, &req);
                let res = service.call(req);
                async move { Ok(mark_immutable(res.await?, named)) }
            }),
    );
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
        // named by their content - compressed at start and not
        std::fs::write(dest_dir.join("style.Mh5rjJWAfXI.css"), &script)?;
        std::fs::write(dest_dir.join("font.jx-LW7_aanc.woff2"), vec![0; 1000])?;

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

        let cache_control = |response: &actix_web::dev::ServiceResponse| {
            response
                .headers()
                .get(header::CACHE_CONTROL)
                .and_then(|value| value.to_str().ok())
                .map(str::to_string)
        };
        let immutable = Some("public, max-age=31536000, immutable".to_string());
        let root = mount_config.dest_http_root();
        for (file, accept_encoding) in [
            ("style.Mh5rjJWAfXI.css", "br"),
            ("style.Mh5rjJWAfXI.css", "gzip"),
            ("font.jx-LW7_aanc.woff2", "br"),
        ] {
            let response =
                call_service(&app, get(&format!("{root}/{file}"), accept_encoding)).await;
            assert_eq!(response.status(), StatusCode::OK, "{file}");
            assert_eq!(
                cache_control(&response),
                immutable,
                "{file}, {accept_encoding}"
            );
        }
        // the same on revalidation of a copy from before
        let response =
            call_service(&app, get(&format!("{root}/style.Mh5rjJWAfXI.css"), "br")).await;
        let etag = response.headers().get(header::ETAG).cloned();
        let Some(etag) = etag else {
            panic!("a build file should have an ETag");
        };
        let revalidate = TestRequest::get()
            .uri(&format!("{root}/style.Mh5rjJWAfXI.css"))
            .insert_header((header::ACCEPT_ENCODING, "br"))
            .insert_header((header::IF_NONE_MATCH, etag))
            .to_request();
        let response = call_service(&app, revalidate).await;
        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
        assert_eq!(cache_control(&response), immutable);

        // a file without the hash, and a missing one, may change
        let response = call_service(&app, get(&script_uri, "br")).await;
        assert_eq!(cache_control(&response), None);
        let response = call_service(&app, get(&format!("{root}/index.json"), "br")).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(cache_control(&response), None);
        // a missing one goes to the page, as before
        let response = call_service(&app, get(&format!("{root}/gone.Mh5rjJWAfXI.css"), "br")).await;
        assert_eq!(cache_control(&response), None);
        assert_eq!(read_body(response).await, "page".as_bytes());

        std::fs::remove_dir_all(&dest_dir)
    }
}
