use actix_cors::Cors;
use actix_web::{App, HttpServer, rt::System, web};
use notify::RecursiveMode;
use std::{path::Path, process::exit, sync::Arc, time::Duration};
use tokio::{
    sync::{Notify, watch::Sender},
    time::sleep,
};
use tokio_retry::{Retry, strategy::FibonacciBackoff};

use crate::build::{Workspace, get_workspace};
use crate::commons::{
    ErrorCode,
    spawn::{SpawnOwner, term_signal},
};

use super::{
    ignore_agent::IgnoreAgents, is_http_server_listening::is_http_server_listening,
    sse::handler_sse, watch_opts::WatchOpts,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Status {
    #[default]
    Building,
    Version(u32),
    Errors,
}

pub async fn run(mut opts: WatchOpts) -> Result<(), ErrorCode> {
    log::info!("watch params => {opts:#?}");

    let ws = match get_workspace() {
        Ok(ws) => ws,
        Err(err) => {
            log::error!("Can't read workspace");
            return Err(err);
        }
    };

    let package_name = match opts.build.package_name.as_deref() {
        Some(name) => name.to_string(),
        None => match ws.infer_package_name() {
            Some(name) => {
                log::info!("Inferred package name = {name}");
                opts.build.package_name = Some(name.clone());
                name
            }
            None => {
                log::error!(
                    "Can't find vertigo project in {} (no cdylib member)",
                    ws.get_root_dir()
                );
                return Err(ErrorCode::CantFindCdylibMember);
            }
        },
    };

    log::info!("package_name ==> {package_name:?}");

    let root = ws.find_package_path(&package_name);
    log::info!("path ==> {root:?}");

    let Some(root) = root else {
        log::error!("package not found ==> {:?}", opts.build.package_name);
        return Err(ErrorCode::PackageNameNotFound);
    };

    // If optimization params not provided, default to false
    if opts.build.release_mode.is_none() {
        opts.build.release_mode = Some(false);
    }
    if opts.build.wasm_opt.is_none() {
        opts.build.wasm_opt = Some(false);
    }

    let excludes = [root.join("target"), root.join(opts.common.dest_dir.clone())];

    let notify_build = Arc::new(Notify::new());

    let watch_result = notify::recommended_watcher({
        let notify_build = notify_build.clone();

        // Generate one Gitignore instance per every watched directory
        let ignore_agents = IgnoreAgents::new(&ws.get_root_dir().into(), &opts);

        move |res: Result<notify::Event, _>| match res {
            Ok(event) => {
                if let notify::EventKind::Access(_) = event.kind {
                    return;
                }

                if event.paths.iter().all(|path| {
                    // Check against hardcoded excludes
                    for exclude_path in &excludes {
                        if path.starts_with(exclude_path) {
                            return true;
                        }
                    }
                    // Check against ignore lists and custom excludes
                    if ignore_agents.should_be_ignored(path) {
                        return true;
                    }
                    false
                }) {
                    return;
                }
                notify_build.notify_one();
            }
            Err(err) => {
                log::error!("watch error: {err:?}");
            }
        }
    });

    let mut watcher = match watch_result {
        Ok(watcher) => watcher,
        Err(error) => {
            log::error!("error watcher => {error}");
            return Err(ErrorCode::WatcherError);
        }
    };

    let (tx, rx) = tokio::sync::watch::channel(Status::default());
    let tx = Arc::new(tx);

    let watch_server = HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .service(web::resource("/events").route(web::get().to(handler_sse)))
            .app_data(web::Data::new(rx.clone()))
    })
    .workers(1)
    .bind("127.0.0.1:5555")
    .map_err(|err| {
        log::error!("Watch server bind error: {err}");
        ErrorCode::WatcherError
    })?
    .disable_signals()
    .run();

    let watch_handle = watch_server.handle();

    std::thread::spawn(move || System::new().block_on(watch_server));

    use notify::Watcher;
    watcher
        .watch(&root, RecursiveMode::Recursive)
        .map_err(|err| {
            log::error!("Can't watch root dir {}: {err}", root.to_string_lossy());
            ErrorCode::CantAddWatchDir
        })?;

    for watch_path in &opts.add_watch_path {
        match watcher.watch(Path::new(watch_path), RecursiveMode::Recursive) {
            Ok(()) => {
                log::info!("Added `{watch_path}` to watched directories");
            }
            Err(err) => {
                log::error!("Error adding watch dir `{watch_path}`: {err}");
                return Err(ErrorCode::CantAddWatchDir);
            }
        }
    }
    let mut version = 0;

    loop {
        version += 1;

        if let Err(err) = tx.send(Status::Building) {
            log::error!("Can't contact the browser: {err} (Other watch process already running?)");
            return Err(ErrorCode::OtherProcessAlreadyRunning);
        };

        sleep(Duration::from_millis(200)).await;

        log::info!("Build run...");

        let spawn = build_and_watch(version, tx.clone(), &opts, &ws);

        tokio::select! {
            msg = term_signal() => {
                log::info!("{msg} received, shutting down");
                spawn.off();
                watch_handle.stop(true).await;
                return Ok(());
            }
            _ = notify_build.notified() => {
                log::info!("Notify build received, shutting down");
                spawn.off();
            }
        }
    }
}

fn build_and_watch(
    version: u32,
    tx: Arc<Sender<Status>>,
    opts: &WatchOpts,
    ws: &Workspace,
) -> SpawnOwner {
    let opts = opts.clone();
    let ws = ws.clone();
    SpawnOwner::new(async move {
        match crate::build::run_with_ws(opts.to_build_opts(), &ws, true) {
            Ok(()) => {
                log::info!("Build successful.");

                let check_spawn = SpawnOwner::new(async move {
                    let _ = Retry::spawn(
                        FibonacciBackoff::from_millis(100).max_delay(Duration::from_secs(4)),
                        || is_http_server_listening(opts.serve.port),
                    )
                    .await;

                    let Ok(()) = tx.send(Status::Version(version)) else {
                        exit(ErrorCode::WatchPipeBroken as i32)
                    };
                });

                let opts = opts.clone();

                log::info!("Spawning serve command...");
                let (mut serve_params, port_watch) = opts.to_serve_opts();

                if serve_params.inner.threads.is_none() {
                    serve_params.inner.threads = Some(2);
                }

                if let Err(error_code) = crate::serve::run(serve_params, Some(port_watch)).await {
                    log::error!("Error {} while running server", error_code as i32);
                    exit(error_code as i32)
                }

                check_spawn.off();
            }
            Err(_) => {
                log::error!("Build run failed. Waiting for changes...");

                let Ok(()) = tx.send(Status::Errors) else {
                    exit(ErrorCode::WatchPipeBroken as i32)
                };
            }
        };
    })
}
