use actix_files::Files;
use actix_web::{http::StatusCode, web, HttpRequest, HttpResponse};
use std::time::Instant;

use crate::serve::MountPathConfig;

use super::server_state::ServerState;

pub fn install_vertigo(cfg: &mut web::ServiceConfig, mount_config: &MountPathConfig) {
    cfg.service(Files::new(
        &mount_config.dest_http_root(),
        mount_config.dest_dir(),
    ))
    .default_service(web::to(vertigo_handler));
}

async fn vertigo_handler(req: HttpRequest) -> HttpResponse {
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
