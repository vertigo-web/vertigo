use actix_web::{http::StatusCode, web, HttpRequest};
use std::time::Instant;

use crate::serve::MountConfig;

use super::server_state::ServerState;

/// Directly attach SSR mechanism to the actix web server (no static files mounting)
///
/// To root path:
///
/// ```no_run
/// # use actix_web::web;
/// # use vertigo_cli::serve::{MountConfigBuilder, vertigo_handler};
/// # let mount_config = MountConfigBuilder::new("/", "/").build().unwrap();
/// # let app = web::scope("/");
/// app.default_service(vertigo_handler(&mount_config));
/// ```
///
/// To custom mount point:
///
/// ```no_run
/// # use actix_web::web;
/// # use vertigo_cli::serve::{MountConfigBuilder, vertigo_handler};
/// # let mount_config = MountConfigBuilder::new("/", "/").build().unwrap();
/// # let app = web::scope("/");
/// app.service(
///     web::scope(mount_config.mount_point())
///         .default_service(vertigo_handler(&mount_config)),
/// );
/// ```
pub fn vertigo_handler(mount_config: &MountConfig) -> actix_web::Route {
    let mount_point = mount_config.mount_point().to_string();

    web::route().to(move |req: HttpRequest| {
        let mount_point = mount_point.clone();
        async move {
            let state = ServerState::global(&mount_point);
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

            actix_web::HttpResponse::from(response_state)
        }
    })
}
