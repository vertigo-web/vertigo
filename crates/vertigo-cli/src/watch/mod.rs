use clap::Args;
use notify::RecursiveMode;
use poem::http::Method;
use poem::middleware::Cors;
use poem::{Route, get, Server, listener::TcpListener, EndpointExt};
use tokio::sync::Notify;
use tokio::time::sleep;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch::Sender;

use crate::build::BuildOpts;
use crate::build::{infer_package_name, find_package_path};
use crate::serve::ServeOpts;
use crate::spawn::SpawnOwner;
use crate::watch::sse::handler_sse;

mod sse;
mod is_http_server_listening;
use is_http_server_listening::is_http_server_listening;

#[derive(Args, Debug, Clone)]
pub struct WatchOpts {
    pub package_name: Option<String>,
    #[arg(long, default_value_t = {"./build".to_string()})]
    pub dest_dir: String,
    #[arg(long, default_value_t = {"/build".to_string()})]
    pub public_path: String,
    #[arg(long, default_value_t = {"127.0.0.1".into()})]
    pub host: String,
    #[arg(long, default_value_t = {4444})]
    pub port: u16,
    #[arg(long, default_value_t = {5555})]
    pub port_watch: u16,
    #[arg(short, long)]
    pub disable_wasm_opt: bool,
}

impl WatchOpts {
    pub fn to_build_opts(&self) -> BuildOpts {
        BuildOpts {
            package_name: self.package_name.clone(),
            dest_dir: self.dest_dir.clone(),
            public_path: self.public_path.clone(),
            disable_wasm_opt: self.disable_wasm_opt,
        }
    }

    pub fn to_serve_opts(&self) -> ServeOpts {
        ServeOpts {
            dest_dir: self.dest_dir.clone(),
            host: self.host.clone(),
            port: self.port,
            port_watch: Some(self.port_watch),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Status {
    #[default]
    Building,
    Version(u32),
    Errors,
}


pub async fn run(opts: WatchOpts) -> Result<(), i32> {
    let package_name = match opts.package_name.as_deref() {
        Some(name) => name.to_string(),
        None => match infer_package_name() {
            Ok(name) => {
                log::info!("Inferred package name = {}", name);
                name
            },
            Err(err) => {
                log::error!("{}", err);
                return Err(-1)
            },
        },
    };

    log::info!("package_name ==> {package_name:?}");

    let path = find_package_path(&package_name);
    log::info!("path ==> {path:?}");

    let Some(path) = path else {
        log::error!("package not found ==> {:?}", opts.package_name);
        return Err(-1);
    };

    let excludes = [
        path.join("target"),
        path.join(opts.dest_dir.clone()),
    ];

    let notify_build = Arc::new(Notify::new());

    let watch_result = notify::RecommendedWatcher::new({
        let notify_build = notify_build.clone();

        move |res: Result<notify::Event, _>| {
            match res {
                Ok(event) => {
                    // log::info!("NOTIFY!");
                    if event.paths.iter().all(|path| {
                        for exclude_path in &excludes {
                            if path.starts_with(exclude_path) {
                                log::info!("Ignoring path {}", path.display());
                                return true
                            }
                        }
                        false
                    }) {
                        return
                    }
                    log::info!("event: {:?}", event);
                    notify_build.notify_one();
                }
                Err(e) => {
                    log::error!("watch error: {:?}", e);
                }
            }
        }
    }, notify::Config::default().with_poll_interval(std::time::Duration::from_millis(200)));

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
    let mut version = 0;

    loop {
        version += 1;

        let Ok(()) = tx.send(Status::Building) else {
            unreachable!();
        };

        log::info!("build run ...");

        let spawn = build_and_watch(version, tx.clone(), &opts);
        notify_build.notified().await;
        spawn.off();
    }
}


fn build_and_watch(version: u32, tx: Arc<Sender<Status>>, opts: &WatchOpts) -> SpawnOwner {
    let opts = opts.clone();

    SpawnOwner::new(async move {
        sleep(Duration::from_millis(200)).await;

        match crate::build::run(opts.to_build_opts()) {
            Ok(()) => {
                log::info!("build run ok");

                let check_spawn = SpawnOwner::new(async move {
                    loop {
                        let is_open = is_http_server_listening(opts.port).await;
                        if is_open {
                            break;
                        }

                        sleep(Duration::from_millis(100)).await;
                    }

                    let Ok(()) = tx.send(Status::Version(version)) else {
                        unreachable!();
                    };

                });

                let opts = opts.clone();

                log::info!("serve run ...");
                if let Err(errno) = crate::serve::run(opts.to_serve_opts()).await {
                    panic!("Error {errno} running server")
                }

                check_spawn.off();

            },
            Err(code) => {
                log::error!("build run failed, exit code={code}");

                let Ok(()) = tx.send(Status::Errors) else {
                    unreachable!();
                };
            }
        };
    })
}
