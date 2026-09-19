use parking_lot::RwLock;
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::mpsc::{error::TryRecvError, unbounded_channel};
use vertigo::{
    JsJson, JsJsonSerialize,
    dev::{
        command::{CommandForBrowser, ConsoleLogLevel, browser_response},
        command_wire::decode_dom_commands,
    },
};
use wasmtime::{Engine, InstancePre, Module};

use crate::{
    commons::{ErrorCode, spawn::SpawnOwner},
    serve::html::FetchCache,
};

use super::{
    html::HtmlResponse,
    mount_path::MountConfig,
    request_state::RequestState,
    response_state::ResponseState,
    timings::SsrProbe,
    wasm::{HostState, Message, WasmInstance, build_linker},
};

#[cfg(feature = "ssr-timings")]
use super::timings::SsrTimings;

pub fn get_now() -> Duration {
    let start = SystemTime::now();
    match start.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration,
        Err(err) => {
            log::error!("Time went backwards: {err}");
            Duration::from_secs(0)
        }
    }
}

pub type ServerStateMap = HashMap<String, Arc<ServerState>>;

static STATE: OnceLock<Arc<RwLock<ServerStateMap>>> = OnceLock::new();

#[derive(Clone)]
pub struct ServerState {
    engine: Engine,
    /// The module with its imports already resolved by name, once, at startup. Holds the
    /// [`Module`] internally, so there is no separate field for it.
    instance_pre: InstancePre<HostState>,
    /// What `Module::from_binary` cost at startup.
    module_compile: Duration,
    pub mount_config: MountConfig,
    pub port_watch: Option<u16>,
}

impl ServerState {
    pub fn init(mount_config: &MountConfig) -> Result<(), ErrorCode> {
        Self::init_with_watch(mount_config, None)
    }

    pub fn init_with_watch(
        mount_config: &MountConfig,
        port_watch: Option<u16>,
    ) -> Result<(), ErrorCode> {
        let engine = Engine::default();

        let (module, module_compile) = build_module_wasm(&engine, mount_config)?;

        // Import resolution happens here, once - a missing or mistyped import is a startup
        // failure naming the offending import, not a per-request one.
        let instance_pre = build_linker(&engine)?
            .instantiate_pre(&module)
            .map_err(|err| {
                log::error!("WASM import resolution failed: {err:?}");
                ErrorCode::ServeWasmInstanceFailed
            })?;

        let mutex = STATE.get_or_init(|| Arc::new(RwLock::new(ServerStateMap::new())));

        let mut guard = mutex.write();
        guard.insert(
            mount_config.mount_point().to_string(),
            Arc::new(Self {
                engine,
                instance_pre,
                module_compile,
                mount_config: mount_config.clone(),
                port_watch,
            }),
        );

        Ok(())
    }

    /// How long compiling this mount point's wasm module took, at startup.
    pub fn module_compile_time(&self) -> Duration {
        self.module_compile
    }

    pub fn global(mount_point: &str) -> Arc<ServerState> {
        let mutex = STATE.get_or_init(|| Arc::new(RwLock::new(ServerStateMap::new())));

        let guard = mutex.read();

        if let Some(state) = guard.get(mount_point) {
            return state.clone();
        }

        unreachable!();
    }

    pub async fn request(&self, url: &str) -> ResponseState {
        self.request_inner(url, &SsrProbe::new()).await
    }

    /// [`ServerState::request`], with the per-phase breakdown of how the render was spent.
    #[cfg(feature = "ssr-timings")]
    pub async fn request_timed(&self, url: &str) -> (ResponseState, SsrTimings) {
        let probe = SsrProbe::new();
        let mark = probe.start();

        let response = self.request_inner(url, &probe).await;

        let timings = probe.finish(mark, response.body.len());
        (response, timings)
    }

    async fn request_inner(&self, url: &str, probe: &SsrProbe) -> ResponseState {
        let (sender, mut receiver) = unbounded_channel::<Message>();

        let request = RequestState {
            url: url.to_string(),
            env: self.mount_config.env.clone(),
        };

        let fetch = FetchCache::new();

        let instantiate_mark = probe.start();
        let handle_command = Arc::new({
            let sender = sender.clone();
            let probe = probe.clone();

            move |request: RequestState, command| match command {
                CommandForBrowser::FetchCacheGet => {
                    browser_response::FetchCacheGet { data: None }.to_json()
                }
                CommandForBrowser::FetchExec { request, callback } => {
                    sender
                        .send(Message::FetchRequest { callback, request })
                        .inspect_err(|err| log::error!("Error sending FetchRequest: {err}"))
                        .unwrap_or_default();

                    JsJson::Null
                }
                CommandForBrowser::SetStatus { status } => {
                    sender
                        .send(Message::SetStatus(status))
                        .inspect_err(|err| log::error!("Error sending FetchRequest: {err}"))
                        .unwrap_or_default();

                    JsJson::Null
                }
                CommandForBrowser::IsBrowser => {
                    let response = browser_response::IsBrowser { value: false };

                    response.to_json()
                }
                CommandForBrowser::GetDateNow => {
                    let time = get_now().as_millis();

                    let response = browser_response::GetDateNow { value: time as u64 };

                    response.to_json()
                }
                CommandForBrowser::WebsocketRegister {
                    host: _,
                    callback: _,
                } => JsJson::Null,
                CommandForBrowser::WebsocketUnregister { callback: _ } => JsJson::Null,
                CommandForBrowser::WebsocketSendMessage {
                    callback: _,
                    message: _,
                } => JsJson::Null,
                CommandForBrowser::TimerSet {
                    callback,
                    duration,
                    kind: _,
                } => {
                    if duration == 0 {
                        sender
                            .send(Message::SetTimeoutZero { callback })
                            .inspect_err(|err| log::error!("Error sending SetTimeoutZero: {err}"))
                            .unwrap_or_default();
                    }

                    JsJson::Null
                }
                CommandForBrowser::TimerClear { callback: _ } => JsJson::Null,
                CommandForBrowser::LocationCallback {
                    target: _,
                    mode: _,
                    callback: _,
                } => JsJson::Null,
                CommandForBrowser::LocationSet {
                    target: _,
                    mode: _,
                    value: _,
                } => JsJson::Null,
                CommandForBrowser::LocationGet { target: _ } => {
                    let url = request.url.clone();
                    browser_response::LocationGet { value: url }.to_json()
                }
                CommandForBrowser::CookieGet { name: _ } => {
                    browser_response::CookieGet { value: "".into() }.to_json()
                }
                CommandForBrowser::CookieSet {
                    name: _,
                    value: _,
                    expires_in: _,
                } => JsJson::Null,
                CommandForBrowser::CookieJsonGet { name: _ } => browser_response::CookieJsonGet {
                    value: JsJson::Null,
                }
                .to_json(),
                CommandForBrowser::CookieJsonSet {
                    name: _,
                    value: _,
                    expires_in: _,
                } => JsJson::Null,
                CommandForBrowser::GetEnv { name } => {
                    let env_value = request.env(name);

                    browser_response::GetEnv { value: env_value }.to_json()
                }
                CommandForBrowser::Log {
                    kind,
                    message,
                    arg2: _,
                    arg3: _,
                    arg4: _,
                } => {
                    if kind == ConsoleLogLevel::Error {
                        log::warn!("{message}");
                    } else {
                        log::info!("{message}");
                    }

                    JsJson::Null
                }
                CommandForBrowser::TimezoneOffset => {
                    browser_response::TimezoneOffset { value: 0 }.to_json()
                }
                CommandForBrowser::HistoryBack => JsJson::Null,
                CommandForBrowser::GetRandom { min, max: _ } => {
                    browser_response::GetRandom { value: min }.to_json()
                }
                CommandForBrowser::JsApiCall { commands: _ } => JsJson::Null,
                CommandForBrowser::DomBulkUpdate { commands } => {
                    // Host work, but reached from inside a wasm call - so this is
                    // phase-3 time measured within a phase-2 region, and `SsrTimings`
                    // subtracts it back out. See the module docs in `timings.rs`.
                    let blob_bytes = commands.len();
                    let decode_mark = probe.start();

                    match decode_dom_commands(&commands) {
                        Ok(list) => {
                            // Before the send, so the channel push is not counted as
                            // decoding.
                            probe.decoded(decode_mark, blob_bytes, list.len());

                            sender
                                .send(Message::DomUpdate(list))
                                .inspect_err(|err| log::error!("Error sending DomUpdate: {err}"))
                                .unwrap_or_default();
                        }
                        Err(err) => log::error!("Error decoding DomBulkUpdate: {err}"),
                    }

                    JsJson::Null
                }
            }
        });

        let mut inst = WasmInstance::new(
            &self.engine,
            &self.instance_pre,
            HostState {
                request,
                sender: sender.clone(),
                probe: probe.clone(),
                handle_command,
            },
        );
        probe.instantiate(instantiate_mark);

        // -- !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
        //TODO - ultimately, do not call call_vertigo_entry_function if something is returned by handle_url
        // -- !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

        inst.call_vertigo_entry_function();

        if let Some(result) = inst.handle_url(url) {
            return result;
        }

        let spawn_resource = SpawnOwner::new({
            let sender = sender.clone();

            async move {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                let _ = sender.send(Message::TimeoutAndSendResponse);
            }
        });

        let mut html_response = HtmlResponse::new(
            sender.clone(),
            &self.mount_config,
            inst,
            self.mount_config.env.clone(),
            fetch,
            probe.clone(),
        );

        loop {
            let message = receiver.try_recv();

            match message {
                Ok(message) => {
                    if let Some(response) = html_response.process_message(message) {
                        return response;
                    };
                    continue;
                }
                Err(TryRecvError::Empty) => {} // continue this iteration
                Err(TryRecvError::Disconnected) => {
                    break; // send response to browser
                }
            }

            if html_response.awaiting_response() {
                // Parked on an SSR fetch: time the request spent, but not time it spent
                // working. Kept in its own bucket so it cannot be read as either.
                let wait_mark = probe.start();
                let message = receiver.recv().await;
                probe.fetch_wait(wait_mark);

                if let Some(message) = message
                    && let Some(response) = html_response.process_message(message)
                {
                    return response;
                };
            } else {
                break; // send response to browser
            }
        }

        spawn_resource.off();
        html_response.build_response()
    }
}

fn build_module_wasm(
    engine: &Engine,
    mount_path: &MountConfig,
) -> Result<(Module, Duration), ErrorCode> {
    let full_wasm_path = mount_path.get_wasm_fs_path();

    log::info!("Mounting {} -> {full_wasm_path}", mount_path.mount_point());

    let wasm_content = match std::fs::read(&full_wasm_path) {
        Ok(wasm_content) => wasm_content,
        Err(error) => {
            log::error!("Problem reading the path: wasm_path={full_wasm_path}, error={error}");
            return Err(ErrorCode::ServeWasmReadFailed);
        }
    };

    let now = Instant::now();

    let module = match Module::from_binary(engine, &wasm_content) {
        Ok(module) => module,
        Err(err) => {
            log::error!("Wasm compilation error: error={err}");
            return Err(ErrorCode::ServeWasmCompileFailed);
        }
    };

    let elapsed = now.elapsed();
    log::info!("WASM module compiled in {} ms.", elapsed.as_millis());
    Ok((module, elapsed))
}
