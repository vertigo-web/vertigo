use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::{error::TryRecvError, unbounded_channel};
use vertigo::{AutoJsJson, BrowserCommand};
use wasmtime::{Engine, Module};

use crate::{commons::{ErrorCode, spawn::SpawnOwner}, serve::html::FetchCache};

use super::{
    html::HtmlResponse,
    mount_path::MountPathConfig,
    request_state::RequestState,
    response_state::ResponseState,
    wasm::{Message, WasmInstance},
};

#[derive(Clone)]
pub struct ServerState {
    engine: Engine,
    module: Module,
    pub mount_config: MountPathConfig,
    pub port_watch: Option<u16>,
    pub env: HashMap<String, String>,
}

impl ServerState {
    pub fn new(
        mount_config: MountPathConfig,
        port_watch: Option<u16>,
        env: Vec<(String, String)>,
    ) -> Result<Self, ErrorCode> {
        let engine = Engine::default();

        let module = build_module_wasm(&engine, &mount_config)?;

        let env = env.into_iter().collect::<HashMap<_, _>>();

        Ok(Self {
            engine,
            module,
            mount_config,
            port_watch,
            env,
        })
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

                move |command| {

                    match command {
                        BrowserCommand::FetchCacheGet => {

                            #[derive(AutoJsJson)]
                            struct Response {
                                data: Option<String>,
                            }

                            let response = Response {
                                data: None,
                            };

                            use vertigo::JsJsonSerialize;
                            return response.to_json();
                        }
                    }
                }
            }));

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

        let mut html_response =
            HtmlResponse::new(sender.clone(), &self.mount_config, inst, self.env.clone(), fetch);

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

            if html_response.waiting_request() {
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
