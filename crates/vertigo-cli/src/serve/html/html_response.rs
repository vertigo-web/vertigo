use crate::serve::{
    html::{fetch_cache::FetchCache, html_build_response::build_response},
    mount_path::MountPathConfig,
    response_state::ResponseState,
    wasm::{Message, WasmInstance},
};
use axum::http::StatusCode;
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;
use vertigo::JsValue;

use super::{
    dom_command::dom_command_from_js_json, element::AllElements, send_request::send_request,
    DomCommand,
};

pub struct HtmlResponse {
    sender: UnboundedSender<Message>,
    mount_path: MountPathConfig,
    inst: WasmInstance,
    all_elements: AllElements,
    fetch: Arc<RwLock<FetchCache>>,
    env: HashMap<String, String>,
    status: StatusCode,
}

impl HtmlResponse {
    pub fn new(
        sender: UnboundedSender<Message>,
        mount_path: &MountPathConfig,
        inst: WasmInstance,
        env: HashMap<String, String>,
        fetch: Arc<RwLock<FetchCache>>,
    ) -> Self {
        Self {
            sender,
            mount_path: mount_path.clone(),
            inst,
            all_elements: AllElements::new(),
            fetch,
            env,
            status: StatusCode::default(),
        }
    }

    pub fn feed(&mut self, commands: Vec<DomCommand>) {
        self.all_elements.feed(commands);
    }

    pub fn awaiting_response(&self) -> bool {
        let guard = self.fetch.read();
        !guard.fetch_waiting.is_empty()
    }

    pub fn build_response(&self) -> ResponseState {
        build_response(
            &self.all_elements,
            &self.env,
            &self.mount_path,
            self.status,
            &self.fetch,
        )
    }

    pub fn process_message(&mut self, message: Message) -> Option<ResponseState> {
        match message {
            Message::TimeoutAndSendResponse => {
                log::info!("timeout");
                Some(self.build_response())
            }
            Message::DomUpdate(update) => {
                match dom_command_from_js_json(update) {
                    Ok(commands) => {
                        self.feed(commands);
                    }
                    Err(message) => {
                        log::error!("DomUpdate: {message}");
                    }
                }

                None
            }
            Message::Panic(message) => {
                let message =
                    message.unwrap_or_else(|| "panic message decoding problem".to_string());
                Some(ResponseState::internal_error(message))
            }
            Message::SetTimeoutZero { callback } => {
                let result = self.inst.wasm_callback(callback, JsValue::Undefined);
                assert_eq!(result, JsValue::Undefined);
                None
            }
            Message::FetchRequest {
                request,
                callback,
            } => {
                let mut guard = self.fetch.write();

                if let Some(response) = guard.fetch_cache.get(&request) {
                    self.inst.send_fetch_response(callback, response.clone());
                    return None;
                }

                if let Some(callbacks) = guard.fetch_waiting.get_mut(&request) {
                    callbacks.push(callback);
                } else {
                    tokio::spawn({
                        let request = request.clone();
                        let sender = self.sender.clone();

                        async move {
                            let response = send_request(request.clone()).await;

                            sender
                                .send(Message::FetchResponse { request, response })
                                .inspect_err(|err| {
                                    log::error!("Error sending fetch response: {err}")
                                })
                                .unwrap_or_default()
                        }
                    });

                    guard.fetch_waiting.insert(request, vec![callback]);
                }
                None
            }

            Message::FetchResponse { request, response } => {
                let mut guard = self.fetch.write();

                let exist = guard.fetch_cache.insert(request.clone(), response.clone());
                assert!(exist.is_none());

                let callback_list = guard.fetch_waiting.remove(&request);

                let Some(callback_list) = callback_list else {
                    unreachable!();
                };

                for callback_id in callback_list {
                    self.inst.send_fetch_response(callback_id, response.clone());
                }

                None
            }

            Message::SetStatus(status) => {
                match StatusCode::from_u16(status) {
                    Ok(status) => self.status = status,
                    Err(err) => log::error!("Invalid status code requested: {err}"),
                }
                None
            }
        }
    }
}
