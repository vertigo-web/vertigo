use crate::serve::html::{DomCommand, HtmlResponse};
use crate::serve::request_state::RequestState;
use crate::serve::spawn::SpawnOwner;
use crate::serve::wasm::{Message, WasmInstance};
use axum::response::Html;

use axum::http::StatusCode;
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
}

impl ServerState {
    pub fn new(mount_path: MountPathConfig) -> Result<Self, i32> {
        let engine = Engine::default();

        let module = build_module_wasm(&engine, &mount_path)?;

        Ok(Self {
            engine,
            module,
            mount_path,
        })
    }

    pub async fn request(&self, url: &str) -> (StatusCode, Html<String>) {
        let (sender, mut receiver) = unbounded_channel::<Message>();

        let request = RequestState {
            url: url.to_string(),
            count: 0,
            name: "name ...".to_string(),
        };

        let mut inst = WasmInstance::new(sender.clone(), &self.engine, &self.module, request);

        inst.call_function("start_application", ());

        let spawn_resource = SpawnOwner::new({
            let sender = sender.clone();

            async move {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                let _ = sender.send(Message::TimeoutAndSendResponse);
            }
        });

        let mut html_response = HtmlResponse::new();

        while let Some(message) = receiver.recv().await {
            match message {
                Message::TimeoutAndSendResponse => {
                    log::info!("timeout");
                    break;
                }
                Message::DomUpdate(update) => {
                    let commands = serde_json::from_str::<Vec<DomCommand>>(update.as_str()).unwrap();
                    html_response.feed(commands);
                }
                Message::Panic(message) => {
                    let message = message.unwrap_or_else(|| "panic message decoding problem".to_string());
                    return (StatusCode::INTERNAL_SERVER_ERROR, Html(message));
                }
                //TODO - callback from api
            }

            if html_response.waiting_request() == 0 {
                break;
            }
        }

        spawn_resource.off();
        (StatusCode::OK, Html(html_response.result(self.mount_path.index.as_slice())))
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
