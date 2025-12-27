use actix_files::Files;
use actix_web::web;

use crate::serve::MountConfig;

use super::vertigo_handler::vertigo_handler;

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
/// To bootstrap a working project using this method, run:
///
/// ```sh
/// vertigo-cli new --fullstack my_project
/// ```
///
pub fn vertigo_install(cfg: &mut web::ServiceConfig, mount_config: &MountConfig) {
    cfg.service(Files::new(
        &mount_config.dest_http_root(),
        mount_config.dest_dir(),
    ))
    .service(web::scope(mount_config.mount_point()).default_service(vertigo_handler(mount_config)));
}
