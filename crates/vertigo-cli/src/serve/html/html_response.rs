use axum::http::StatusCode;
use std::{collections::{HashMap, VecDeque}, sync::Arc};
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
    send_request::send_request,
    dom_command::dom_command_from_js_json,
    html_element::{
        HtmlElement,
        HtmlDocument
    },
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

    pub fn waiting_request(&self) -> u32 {
        let mut count = 0;

        for (_, state) in self.fetch.iter() {
            if state.is_requested() {
                count += 1;
            }
        }

        count
    }

    pub fn build_response(&self) -> (StatusCode, String) {
        let (mut root_html, css) = self.all_elements.get_response(false);

        let css = css.into_iter().collect::<VecDeque<_>>();

        let root_ok = if let Some(element) = root_html.get_element() {
            element.name == "html"
        } else {
            false
        };

        if !root_ok {
            let message = "root: the html element was expected".into();
            return (StatusCode::INTERNAL_SERVER_ERROR, message);
        }

        let is_exist_head = root_html.modify(&[("head", 0)], move |_head| {});

        if !is_exist_head {
            let message = "The 'head' element was expected in the response".into();
            return (StatusCode::INTERNAL_SERVER_ERROR, message);
        }

        let script = HtmlElement::new("script")
            .attr("type", "module")
            .attr("data-vertigo-run-wasm", &self.mount_path.wasm_path)
            .attr("src", &self.mount_path.run_js);

        let success = root_html.modify(&[("body", 0)], move |body| {
            for css_node in css.into_iter().rev() {
                body.add_first_child(css_node);
            }

            body.add_last_child(script);
        });

        if success {
            let document = HtmlDocument::new(root_html);
            (StatusCode::OK, document.convert_to_string(true))
        } else {
            let message = "The 'body' element was expected in the response".into();
            (StatusCode::INTERNAL_SERVER_ERROR, message)
        }
    }

    pub fn process_message(&mut self, message: Message) -> Option<(StatusCode, String)> {
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
                Some((StatusCode::INTERNAL_SERVER_ERROR, message))
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
