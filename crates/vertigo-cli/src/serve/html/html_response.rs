use std::{collections::HashMap, sync::Arc};

use axum::{http::StatusCode, response::Html};
use tokio::sync::mpsc::UnboundedSender;
use crate::serve::{
    wasm::{
        Message,
        WasmInstance,
        FetchRequest,
        FetchResponse
    },
    mount_path::MountPathConfig,
    js_value::JsValue
};

use super::{
    DomCommand,
    element::AllElements,
    replace_html,
    send_request::send_request,
    dom_command::dom_command_from_js_json,
    html_element::{
        HtmlElement,
        HtmlDocument
    },
    HtmlNode
};

enum FetchStatus {
    Requested {
        callbacks: Vec<u64>,
    },
    Response {
        response: FetchResponse,
    }
}

impl FetchStatus {
    fn is_requested(&self) -> bool {
        matches!(self, Self::Requested { .. })
    }
}

pub struct HtmlResponse {
    sender: UnboundedSender<Message>,
    mount_path: MountPathConfig,
    inst: WasmInstance,
    all_elements: AllElements,
    fetch: HashMap<Arc<FetchRequest>, FetchStatus>,
}

impl HtmlResponse {
    pub fn new(sender: UnboundedSender<Message>, mount_path: &MountPathConfig, inst: WasmInstance) -> Self {
        Self {
            sender,
            mount_path: mount_path.clone(),
            inst,
            all_elements: AllElements::new(),
            fetch: HashMap::new(),
        }
    }

    pub fn feed(&mut self, commands: Vec<DomCommand>) {
        self.all_elements.feed(commands);
    }

    pub fn result(&self) -> HtmlDocument {
        let index = self.mount_path.index.as_ref();

        let content = self.all_elements.get_response_nodes(false);

        let get_content = move || -> Vec<HtmlNode> {
            content.clone()
        };

        replace_html(index, &test_node, &get_content)
    }

    pub fn waiting_request(&self) -> u32 {
        let mut count = 0;

        for (_, state) in self.fetch.iter() {
            if state.is_requested() {
                count += 1;
            }
        }

        count
    }

    pub fn build_response(&self) -> (StatusCode, Html<String>) {
        (StatusCode::OK, Html(self.result().convert_to_string(true)))
    }

    pub fn process_message(&mut self, message: Message) -> Option<(StatusCode, Html<String>)> {
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
                let message = message.unwrap_or_else(|| "panic message decoding problem".to_string());
                Some((StatusCode::INTERNAL_SERVER_ERROR, Html(message)))
            }
            Message::SetTimeoutZero { callback_id } => {
                let result = self.inst.wasm_callback(callback_id, JsValue::Undefined);
                assert_eq!(result, JsValue::Undefined);
                None
            },
            Message::FetchRequest { callback_id, request } => {
                let request = Arc::new(request);

                if let Some(value) = self.fetch.get_mut(&request) {
                    match value {
                        FetchStatus::Requested { callbacks } => {
                            callbacks.push(callback_id);
                        },
                        FetchStatus::Response { response } => {
                            self.inst.send_fetch_response(callback_id, response.clone());
                        }
                    }
                } else {
                    tokio::spawn({
                        let request = request.clone();
                        let sender = self.sender.clone();

                        async move {
                            let response = send_request(request.clone()).await;
                            
                            sender.send(Message::FetchResponse {
                                request,
                                response
                            }).unwrap();
                        }
                    });

                    self.fetch.insert(request, FetchStatus::Requested {
                        callbacks: vec!(callback_id),
                    });
                }
                None
            },

            Message::FetchResponse { request, response } => {
                let state = self.fetch.remove(&request);

                let new_state = match state {
                    Some(state) => {
                        match state {
                            FetchStatus::Requested { callbacks } => {
                                for callback_id in callbacks {
                                    self.inst.send_fetch_response(callback_id, response.clone());
                                }
                                FetchStatus::Response { response }
                            },
                            FetchStatus::Response { .. } => {
                                unreachable!();
                            }
                        }
                    },
                    None => {
                        FetchStatus::Response { response }
                    }
                };

                self.fetch.insert(request, new_state);
    
                None
            },
        }
    }

}

fn test_node(node: &HtmlElement) -> bool {
    node.attr.contains_key("data-vertigo-run-wasm")
}