use std::collections::HashMap;
use tokio::sync::mpsc::{error::TryRecvError, unbounded_channel};
use wasmtime::{Engine, Module};

use crate::commons::{spawn::SpawnOwner, ErrorCode};

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
    pub mount_path: MountPathConfig,
    pub port_watch: Option<u16>,
    pub env: HashMap<String, String>,
}

impl ServerState {
    pub fn new(
        mount_path: MountPathConfig,
        port_watch: Option<u16>,
        env: Vec<(String, String)>,
    ) -> Result<Self, ErrorCode> {
        let engine = Engine::default();

        let module = build_module_wasm(&engine, &mount_path)?;

        let env = env.into_iter().collect::<HashMap<_, _>>();

        Ok(Self {
            engine,
            module,
            mount_path,
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

        let mut inst = WasmInstance::new(sender.clone(), &self.engine, &self.module, request);

        inst.call_vertigo_entry_function();

        let spawn_resource = SpawnOwner::new({
            let sender = sender.clone();

            async move {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                let _ = sender.send(Message::TimeoutAndSendResponse);
            }
        });

        let mut html_response =
            HtmlResponse::new(sender.clone(), &self.mount_path, inst, self.env.clone());

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

            if html_response.waiting_request() == 0 {
                //send response to browser
                break;
            } else {
                let message = receiver.recv().await;
                if let Some(message) = message {
                    if let Some(response) = html_response.process_message(message) {
                        return response;
                    };
                }
            }
        }

        spawn_resource.off();
        html_response.build_response()
    }
}

fn build_module_wasm(engine: &Engine, mount_path: &MountPathConfig) -> Result<Module, ErrorCode> {
    let full_wasm_path = mount_path.translate_to_fs(&mount_path.wasm_path)?;

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
