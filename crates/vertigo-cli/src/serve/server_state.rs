use crate::serve::html::HtmlResponse;
use crate::serve::request_state::RequestState;
use crate::serve::spawn::SpawnOwner;
use crate::serve::wasm::{Message, WasmInstance};

use axum::http::StatusCode;
use tokio::sync::mpsc::error::TryRecvError;
use wasmtime::{
    Engine,
    Module,
};
use tokio::sync::mpsc::{unbounded_channel};
use super::mount_path::MountPathConfig;

#[derive(Clone)]
pub struct ServerState {
    engine: Engine,
    module: Module,
    pub mount_path: MountPathConfig,
    pub port_watch: Option<u16>,
}

impl ServerState {
    pub fn new(mount_path: MountPathConfig, port_watch: Option<u16>) -> Result<Self, i32> {
        let engine = Engine::default();

        let module = build_module_wasm(&engine, &mount_path)?;

        Ok(Self {
            engine,
            module,
            mount_path,
            port_watch,
        })
    }

    pub async fn request(&self, url: &str) -> (StatusCode, String) {
        let (sender, mut receiver) = unbounded_channel::<Message>();

        let request = RequestState {
            url: url.to_string(),
        };

        let mut inst = WasmInstance::new(sender.clone(), &self.engine, &self.module, request);

        inst.call_start_application();

        let spawn_resource = SpawnOwner::new({
            let sender = sender.clone();

            async move {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                let _ = sender.send(Message::TimeoutAndSendResponse);
            }
        });

        let mut html_response = HtmlResponse::new(sender.clone(), &self.mount_path, inst);

        loop {
            let message = receiver.try_recv();
            
            match message {
                Ok(message) => {
                    if let Some(response) = html_response.process_message(message) {
                        return response;
                    };
                    continue;
                },
                Err(TryRecvError::Empty) => {
                    //continue this iteration
                },
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

fn build_module_wasm(engine: &Engine, mount_path: &MountPathConfig) -> Result<Module, i32> {
    let full_wasm_path = mount_path.translate_to_fs(&mount_path.wasm_path)?;

    log::info!("full_wasm_path = {full_wasm_path}");

    let wasm_content = match std::fs::read(&full_wasm_path) {
        Ok(wasm_content) => wasm_content,
        Err(error) => {
            log::error!("Problem reading the path: wasm_path={full_wasm_path}, error={error}");
            return Err(-1);
        }
    };

    let module = match Module::from_binary(engine, &wasm_content) {
        Ok(module) => module,
        Err(err) => {
            log::error!("Wasm compilation error: error={err}");
            return Err(-1);
        }
    };

    Ok(module)
}
