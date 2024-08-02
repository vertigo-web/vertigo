use notify::RecursiveMode;
use poem::http::Method;
use poem::middleware::Cors;
use poem::{get, listener::TcpListener, EndpointExt, Route, Server};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch::Sender;
use tokio::sync::Notify;
use tokio::time::sleep;
use tokio_retry::{strategy::FibonacciBackoff, Retry};

use crate::build::{get_workspace, Workspace};
use crate::commons::spawn::SpawnOwner;
use crate::watch::sse::handler_sse;

use super::is_http_server_listening::is_http_server_listening;
use super::watch_opts::WatchOpts;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Status {
    #[default]
    Building,
    Version(u32),
    Errors,
}

pub async fn run(mut opts: WatchOpts) -> Result<(), i32> {
    log::info!("watch params => {opts:#?}");

    let ws = get_workspace().expect("Can't read workspace");

    let package_name = match opts.build.package_name.as_deref() {
        Some(name) => name.to_string(),
        None => match ws.infer_package_name() {
            Some(name) => {
                log::info!("Inferred package name = {}", name);
                opts.build.package_name = Some(name.clone());
                name
            }
            None => {
                log::error!(
                    "Can't find vertigo project in {} (no cdylib member)",
                    ws.get_root_dir()
                );
                return Err(-1);
            }
        },
    };

    log::info!("package_name ==> {package_name:?}");

    let path = ws.find_package_path(&package_name);
    log::info!("path ==> {path:?}");

    let Some(path) = path else {
        log::error!("package not found ==> {:?}", opts.build.package_name);
        return Err(-1);
    };

    let excludes = [path.join("target"), path.join(opts.common.dest_dir.clone())];

    let notify_build = Arc::new(Notify::new());

    let watch_result = notify::RecommendedWatcher::new(
        {
            let notify_build = notify_build.clone();

            move |res: Result<notify::Event, _>| match res {
                Ok(event) => {
                    if event.paths.iter().all(|path| {
                        for exclude_path in &excludes {
                            if path.starts_with(exclude_path) {
                                return true;
                            }
                        }
                        false
                    }) {
                        return;
                    }
                    log::info!("event: {:?}", event);
                    notify_build.notify_one();
                }
                Err(e) => {
                    log::error!("watch error: {:?}", e);
                }
            }
        },
        notify::Config::default().with_poll_interval(std::time::Duration::from_millis(200)),
    );

    let mut watcher = match watch_result {
        Ok(watcher) => watcher,
        Err(error) => {
            log::error!("error watcher => {error}");
            return Err(-1);
        }
    };

    let (tx, rx) = tokio::sync::watch::channel(Status::default());
    let tx = Arc::new(tx);

    tokio::spawn({
        let cors_middleware = Cors::new()
            .allow_methods(vec![Method::GET, Method::POST])
            .max_age(3600);

        let app = Route::new()
            .at("/events", get(handler_sse))
            .with(cors_middleware)
            .data(rx);

        async move {
            Server::new(TcpListener::bind("127.0.0.1:5555"))
                .run(app)
                .await
        }
    });

    use notify::Watcher;
    watcher.watch(&path, RecursiveMode::Recursive).unwrap();

    for watch_path in &opts.add_watch_path {
        log::info!("Adding `{watch_path}` to watched directories");
        watcher
            .watch(Path::new(watch_path), RecursiveMode::Recursive)
            .expect("Error adding watch dir");
    }
    let mut version = 0;

    loop {
        version += 1;

        if let Err(err) = tx.send(Status::Building) {
            log::error!("Can't contact the browser: {err} (Other watch process already running?)");
            return Err(-2);
        };

        log::info!("build run ...");

        let spawn = build_and_watch(version, tx.clone(), &opts, &ws);
        notify_build.notified().await;
        spawn.off();
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
        sleep(Duration::from_millis(200)).await;

        match crate::build::run_with_ws(opts.to_build_opts(), &ws) {
            Ok(()) => {
                log::info!("build run ok");

                let check_spawn = SpawnOwner::new(async move {
                    let _ = Retry::spawn(
                        FibonacciBackoff::from_millis(100).max_delay(Duration::from_secs(4)),
                        || is_http_server_listening(opts.serve.port),
                    )
                    .await;

                    let Ok(()) = tx.send(Status::Version(version)) else {
                        unreachable!();
                    };
                });

                let opts = opts.clone();

                log::info!("serve run ...");
                let (serve_params, port_watch) = opts.to_serve_opts();

                if let Err(errno) = crate::serve::run(serve_params, Some(port_watch)).await {
                    panic!("Error {errno} running server")
                }

                check_spawn.off();
            }
            Err(code) => {
                log::error!("build run failed, exit code={code}");

                let Ok(()) = tx.send(Status::Errors) else {
                    unreachable!();
                };
            }
        };
    })
}
