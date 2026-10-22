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

    if mount_config.precompress {
        let files = ServerState::global(mount_point).precompressed();
        cfg.service(precompressed_files(&mount_config.dest_http_root(), files));
    }

    cfg.service(Files::new(
        &mount_config.dest_http_root(),
        mount_config.dest_dir(),
    ));

    if mount_point == "/" {
        cfg.default_service(vertigo_handler(mount_config));
    } else {
        cfg.service(web::scope(mount_point).default_service(vertigo_handler(mount_config)));
    };
}
