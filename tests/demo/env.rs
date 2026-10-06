//! Bring the demo up for the browser, and take it down again.
//!
//! Built and served the way the demo ships: a release build, and `vertigo serve` itself, with
//! the proxy, the env and compression - not the harness's plain in-process server, which has
//! no proxy. On free ports, so it can run next to the other suites.
//!
//! What is extra here is the API server - the demo has a backend, and the tabs that use it are
//! the interesting ones.

use std::{ops::Deref, sync::Mutex, time::Duration};

use tokio::sync::oneshot;
use vertigo_cli::{CommonOpts, ServeOpts, serve};
use vertigo_testing::{ChromeConfig, Core, Env, Settings, build, free_port, prelude::*};

const PACKAGE: &str = "vertigo-demo";

pub struct DemoEnv {
    core: Core,
    /// The demo's own API - `/api/items`, the two websockets, and the stubs standing in for
    /// jsonplaceholder and GitHub.
    api: vertigo_demo_server::ServerHandle,
    serve_stop: Mutex<Option<oneshot::Sender<()>>>,
}

impl Env for DemoEnv {
    async fn start(suite: &'static str) -> Result<Self> {
        let mut settings = Settings::from_env(env!("CARGO_MANIFEST_DIR"))?;
        // Release, so what is tested is what ships. Debug builds also emit extra `v-component`
        // and `v-css` attributes, which would show up in the class assertions.
        settings.release = true;
        settings.wasm_opt = true;

        let api_port = free_port()?;
        println!("Starting the demo API server on port {api_port}");
        let api = vertigo_demo_server::start_background("127.0.0.1", api_port)
            .context("could not start the demo API server")?;

        let build_dir = settings.build_dir().join("demo");
        build::vertigo_app(&settings, PACKAGE, &build_dir).await?;

        let port = free_port()?;
        println!("Spawning vertigo serve on port {port}");
        let serve_stop = spawn_serve(&build_dir.to_string_lossy(), port, api_port);
        wait_for_listener(port).await?;

        let core = Core::new(
            suite,
            settings,
            format!("http://127.0.0.1:{port}"),
            // Wide enough that the demo's flex rows are not permanently squeezed.
            //
            // A default WebDriver window is around 800px, and the Sudoku tab alone wants 676 of
            // them for its board - so the panel beside it sits at its minimum width and
            // *cannot* grow, whatever is put in it. That hides the reflow the layout
            // assertions exist to catch, and would let one of them pass while the bug it
            // names was still there.
            ChromeConfig {
                window_size: (1400, 1000),
                ..Default::default()
            },
        )?;

        Ok(Self {
            core,
            api,
            serve_stop: Mutex::new(Some(serve_stop)),
        })
    }

    async fn shutdown(&self) {
        let stop = self
            .serve_stop
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(stop) = stop {
            let _ = stop.send(());
        }
        self.api.stop(false).await;
    }
}

impl Deref for DemoEnv {
    type Target = Core;

    fn deref(&self) -> &Core {
        &self.core
    }
}

/// `vertigo serve` on a thread of its own, stopped by the returned sender.
fn spawn_serve(dest_dir: &str, port: u16, api_port: u16) -> oneshot::Sender<()> {
    let api = format!("http://127.0.0.1:{api_port}");
    let opts = ServeOpts {
        common: CommonOpts {
            dest_dir: dest_dir.to_string(),
            log_local_time: None,
        },
        inner: serve::ServeOptsInner {
            host: "127.0.0.1".into(),
            port,
            mount_point: "/".to_string(),
            // The lazy-list tab asks for a relative `/api/items`, so that one has to arrive
            // same-origin.
            proxy: vec![("/api".to_string(), format!("{api}/api"))],
            env: vec![
                // Websockets go direct: `install_proxy` forwards with `awc` and does not do
                // upgrade handshakes.
                (
                    "ws_chat".to_string(),
                    format!("ws://127.0.0.1:{api_port}/ws"),
                ),
                (
                    "ws_collection".to_string(),
                    format!("ws://127.0.0.1:{api_port}/ws-collection"),
                ),
                // The two public APIs, pointed at their local stand-ins.
                ("api_fetch".to_string(), format!("{api}/fetch")),
                ("api_github".to_string(), format!("{api}/github")),
            ],
            wasm_preload: true,
            disable_hydration: false,
            ssr_fetch_base: None,
            // Left on, as a served app has it.
            disable_compression: false,
            threads: None,
        },
    };

    let (stop, stopped) = oneshot::channel::<()>();
    let handle = tokio::runtime::Handle::current();
    std::thread::spawn(move || {
        handle.block_on(async {
            tokio::select! {
                ret = serve::run(opts, None) => {
                    if let Err(err) = ret {
                        eprintln!("Can't spawn vertigo-cli: {err:?}");
                    }
                }
                _ = stopped => {}
            }
        });
    });

    stop
}

/// Waits until something accepts connections on `port`. `serve::run` compiles the wasm for SSR
/// before it binds.
async fn wait_for_listener(port: u16) -> Result<()> {
    let deadline = std::time::Instant::now() + Duration::from_secs(60);

    loop {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            return Ok(());
        }
        ensure!(
            std::time::Instant::now() < deadline,
            "nothing came up on port {port}"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
