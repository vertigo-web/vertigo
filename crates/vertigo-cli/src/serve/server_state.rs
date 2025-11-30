use parking_lot::RwLock;
use std::sync::OnceLock;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::mpsc::{error::TryRecvError, unbounded_channel};
use vertigo::{
    dev::command::{browser_response, CommandForBrowser, ConsoleLogLevel},
    JsJson, JsJsonSerialize,
};
use wasmtime::{Engine, Module};

use crate::{
    commons::{spawn::SpawnOwner, ErrorCode},
    serve::html::FetchCache,
};

use super::{
    html::HtmlResponse,
    mount_path::MountPathConfig,
    request_state::RequestState,
    response_state::ResponseState,
    wasm::{Message, WasmInstance},
};

pub fn get_now() -> Duration {
    let start = SystemTime::now();
    start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
}

static STATE: OnceLock<Arc<RwLock<Option<Arc<ServerState>>>>> = OnceLock::new();

#[derive(Clone)]
pub struct ServerState {
    engine: Engine,
    module: Module,
    pub mount_config: MountPathConfig,
    pub port_watch: Option<u16>,
    pub env: HashMap<String, String>,
}

impl ServerState {
    pub fn init(
        mount_config: MountPathConfig,
        port_watch: Option<u16>,
        env: Vec<(String, String)>,
    ) -> Result<(), ErrorCode> {
        let engine = Engine::default();

        let module = build_module_wasm(&engine, &mount_config)?;

        let env = env.into_iter().collect::<HashMap<_, _>>();

        let mutex = STATE.get_or_init(|| Arc::new(RwLock::new(None)));

        let mut guard = mutex.write();
        *guard = Some(Arc::new(Self {
            engine,
            module,
            mount_config,
            port_watch,
            env,
        }));

        Ok(())
    }

    pub fn global() -> Arc<ServerState> {
        let mutex = STATE.get_or_init(|| Arc::new(RwLock::new(None)));

        let guard = mutex.read();

        if let Some(state) = &*guard {
            return state.clone();
        }

        unreachable!();
    }

    pub async fn request(&self, url: &str) -> ResponseState {
        let (sender, mut receiver) = unbounded_channel::<Message>();

        let request = RequestState {
            url: url.to_string(),
            env: self.env.clone(),
        };

        let fetch = FetchCache::new();

        let mut inst = WasmInstance::new(
            sender.clone(),
            &self.engine,
            &self.module,
            request,
            Arc::new({
                let sender = sender.clone();

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
                                .inspect_err(|err| {
                                    log::error!("Error sending SetTimeoutZero: {err}")
                                })
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
                    CommandForBrowser::CookieJsonGet { name: _ } => {
                        browser_response::CookieJsonGet {
                            value: JsJson::Null,
                        }
                        .to_json()
                    }
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
                    CommandForBrowser::DomBulkUpdate { list } => {
                        sender
                            .send(Message::DomUpdate(list))
                            .inspect_err(|err| log::error!("Error sending DomUpdate: {err}"))
                            .unwrap_or_default();

                        JsJson::Null
                    }
                }
            }),
        );

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
            self.env.clone(),
            fetch,
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
                Err(TryRecvError::Empty) => {
                    //continue this iteration
                }
                Err(TryRecvError::Disconnected) => {
                    //send response to browser
                    break;
                }
            }

            if html_response.awaiting_response() {
                let message = receiver.recv().await;
                if let Some(message) = message {
                    if let Some(response) = html_response.process_message(message) {
                        return response;
                    };
                }
            } else {
                //send response to browser
                break;
            }
        }

        spawn_resource.off();
        html_response.build_response()
    }
}

fn build_module_wasm(engine: &Engine, mount_path: &MountPathConfig) -> Result<Module, ErrorCode> {
    let full_wasm_path = mount_path.get_wasm_fs_path();

    log::info!("full_wasm_path = {full_wasm_path}");

    let wasm_content = match std::fs::read(&full_wasm_path) {
        Ok(wasm_content) => wasm_content,
        Err(error) => {
            log::error!("Problem reading the path: wasm_path={full_wasm_path}, error={error}");
            return Err(ErrorCode::ServeWasmReadFailed);
        }
    };

    let module = match Module::from_binary(engine, &wasm_content) {
        Ok(module) => module,
        Err(err) => {
            log::error!("Wasm compilation error: error={err}");
            return Err(ErrorCode::ServeWasmCompileFailed);
        }
    };

    Ok(module)
}
