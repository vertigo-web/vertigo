use axum::http::StatusCode;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
use tokio::sync::mpsc::UnboundedSender;
use vertigo::JsValue;

use crate::serve::{
    mount_path::MountPathConfig,
    response_state::ResponseState,
    wasm::{FetchRequest, FetchResponse, Message, WasmInstance},
};

use super::{
    dom_command::dom_command_from_js_json,
    element::AllElements,
    html_element::{HtmlDocument, HtmlElement},
    send_request::send_request,
    DomCommand, HtmlNode,
};

enum FetchStatus {
    Requested { callbacks: Vec<u64> },
    Response { response: FetchResponse },
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
    env: HashMap<String, String>,
    status: StatusCode,
}

impl HtmlResponse {
    pub fn new(
        sender: UnboundedSender<Message>,
        mount_path: &MountPathConfig,
        inst: WasmInstance,
        env: HashMap<String, String>,
    ) -> Self {
        Self {
            sender,
            mount_path: mount_path.clone(),
            inst,
            all_elements: AllElements::new(),
            fetch: HashMap::new(),
            env,
            status: StatusCode::default(),
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

    pub fn build_response(&self) -> ResponseState {
        let (mut root_html, css) = self.all_elements.get_response(false);

        let css = css.into_iter().collect::<VecDeque<_>>();

        if let HtmlNode::Element(html) = &mut root_html {
            if html.name != "html" {
                // Not really possible
                return ResponseState::internal_error(format!(
                    "Missing <html> element, found {} instead",
                    html.name
                ));
            }

            for (env_name, env_value) in &self.env {
                html.add_attr(format!("data-env-{env_name}"), env_value);
            }
        } else {
            return ResponseState::internal_error("Missing <html> element");
        }

        let head_exists = root_html.modify(&[("head", 0)], move |_head| {});

        if !head_exists {
            log::info!("Missing <head> element");
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
            ResponseState::html(self.status, document.convert_to_string(true))
        } else {
            ResponseState::internal_error("Missing <body> element")
        }
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
            Message::SetTimeoutZero { callback_id } => {
                let result = self.inst.wasm_callback(callback_id, JsValue::Undefined);
                assert_eq!(result, JsValue::Undefined);
                None
            }
            Message::FetchRequest {
                callback_id,
                request,
            } => {
                let request = Arc::new(request);

                if let Some(value) = self.fetch.get_mut(&request) {
                    match value {
                        FetchStatus::Requested { callbacks } => {
                            callbacks.push(callback_id);
                        }
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

                            sender
                                .send(Message::FetchResponse { request, response })
                                .inspect_err(|err| {
                                    log::error!("Error sending fetch response: {err}")
                                })
                                .unwrap_or_default()
                        }
                    });

                    self.fetch.insert(
                        request,
                        FetchStatus::Requested {
                            callbacks: vec![callback_id],
                        },
                    );
                }
                None
            }

            Message::FetchResponse { request, response } => {
                let state = self.fetch.remove(&request);

                let new_state = match state {
                    Some(state) => match state {
                        FetchStatus::Requested { callbacks } => {
                            for callback_id in callbacks {
                                self.inst.send_fetch_response(callback_id, response.clone());
                            }
                            FetchStatus::Response { response }
                        }
                        FetchStatus::Response { .. } => {
                            log::error!("Unreachable in process_message");
                            return None;
                        }
                    },
                    None => FetchStatus::Response { response },
                };

                self.fetch.insert(request, new_state);

                None
            }

            Message::PlainResponse(body) => Some(ResponseState::plain(self.status, body)),

            Message::SetStatus(status) => {
                match StatusCode::from_u16(status) {
                    Ok(status) => self.status = status,
                    Err(err) => log::error!("Invalid status code reqeusted: {err}"),
                }
                None
            }
        }
    }
}
