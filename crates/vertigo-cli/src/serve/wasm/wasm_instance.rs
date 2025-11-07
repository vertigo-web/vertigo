use reqwest::StatusCode;
use std::{collections::HashMap, process::exit};
use tokio::sync::mpsc::UnboundedSender;
use vertigo::{JsValue, LongPtr};
use wasmtime::{Caller, Engine, Func, Instance, Module, Store};

use crate::{
    commons::ErrorCode,
    serve::{request_state::RequestState, response_state::ResponseState},
};

use super::{
    data_context::DataContext,
    decode_commands::*,
    message::{CallWebsocketResult, Message},
    FetchResponse,
};

pub struct WasmInstance {
    instance: Instance,
    store: Store<RequestState>,
}

impl WasmInstance {
    pub fn new(
        sender: UnboundedSender<Message>,
        engine: &Engine,
        module: &Module,
        request: RequestState,
    ) -> Self {
        let url = request.url.clone();
        let mut store = Store::new(engine, request.clone());

        let import_panic_message = Func::wrap(&mut store, {
            let sender = sender.clone();

            move |caller: Caller<'_, RequestState>, long_ptr: u64| {
                let mut data_context = DataContext::from_caller(caller);
                let long_ptr = LongPtr::from(long_ptr);
                let (ptr, offset) = long_ptr.into_parts();

                let message = data_context.get_string_from(ptr, offset);
                log::error!("wasm panic: {message:?}");

                sender.send(Message::Panic(message)).unwrap_or_default();
            }
        });

        let import_dom_access = {
            Func::wrap(
                &mut store,
                move |caller: Caller<'_, RequestState>, long_ptr: u64| -> u64 {
                    let long_ptr = LongPtr::from(long_ptr);
                    let mut data_context = DataContext::from_caller(caller);

                    let value = data_context.get_value_long_ptr(long_ptr);

                    // Ignore cookie operations
                    if let Ok(()) = match_cookie_command(&value) {
                        return 0;
                    }

                    // get history router location
                    if let Ok(()) = match_history_router(&value) {
                        let result = JsValue::str(url.clone());
                        return data_context.save_value(result).get_long_ptr();
                    }

                    if let Ok(env_name) = match_get_env(&value) {
                        let env_value = request.env(env_name);

                        let result = match env_value {
                            Some(value) => JsValue::String(value),
                            None => JsValue::Null,
                        };
                        return data_context.save_value(result).get_long_ptr();
                    }

                    //adding callback for hashrouter
                    if match_history_router_callback(&value).is_ok() {
                        return 0;
                    }

                    if let Ok(data) = match_dom_bulk_update(&value) {
                        sender
                            .send(Message::DomUpdate(data))
                            .inspect_err(|err| log::error!("Error sending DomUpdate: {err}"))
                            .unwrap_or_default();
                        return 0;
                    }

                    if let Ok((log_type, log_message)) = match_log(&value) {
                        if log_type == "error" {
                            log::warn!("{log_message}");
                        } else {
                            log::info!("{log_message}");
                        }
                        return 0;
                    }

                    if let Ok(current_time) = match_date_now(&value) {
                        return data_context.save_value(current_time).get_long_ptr();
                    }

                    if let Ok(result) = match_interval(&value) {
                        match result {
                            CallWebsocketResult::TimeoutSet { time, callback_id } => {
                                if time == 0 {
                                    sender
                                        .send(Message::SetTimeoutZero { callback_id })
                                        .inspect_err(|err| {
                                            log::error!("Error sending SetTimeoutZero: {err}")
                                        })
                                        .unwrap_or_default();
                                }

                                let result = JsValue::I32(0); // fake timerId
                                return data_context.save_value(result).get_long_ptr();
                            }
                            CallWebsocketResult::NoResult => {
                                return 0;
                            }
                        }
                    }

                    if let Ok(()) = match_websocket(&value) {
                        return 0;
                    }

                    if let Ok((callback_id, request)) = match_fetch(&value) {
                        sender
                            .send(Message::FetchRequest {
                                callback_id,
                                request,
                            })
                            .inspect_err(|err| log::error!("Error sending FetchRequest: {err}"))
                            .unwrap_or_default();
                        return 0;
                    }

                    if match_is_browser(&value).is_ok() {
                        return data_context.save_value(JsValue::False).get_long_ptr();
                    }

                    if let Ok(status) = match_is_set_status(&value) {
                        sender
                            .send(Message::SetStatus(status))
                            .inspect_err(|err| log::error!("Error setting status code: {err}"))
                            .unwrap_or_default();
                        return 0;
                    }

                    // push history router location
                    if let Ok(_url) = match_history_router_push(&value) {
                        // Ignore in SSR
                        return 0;
                    }

                    log::error!("unsupported message: {value:#?}");
                    0
                },
            )
        };

        let mut imports = [import_dom_access.into(), import_panic_message.into()];
        let instance = match Instance::new(&mut store, module, &imports) {
            Ok(instance) => instance,
            Err(err) => {
                // Workaround for rust/wasmtime mangling with functions order.
                // Upon error try with panic/dom_access reversed before giving up.
                imports.reverse();
                match Instance::new(&mut store, module, &imports) {
                    Ok(instance) => {
                        log::warn!("WASM instantiation types order problem - update rust or soon it will stop working");
                        instance
                    }
                    Err(err2) => {
                        log::error!("WASM instantiation error (1): {err:?}");
                        log::error!("WASM instantiation error (2): {err2:?}");
                        exit(ErrorCode::ServeWasmInstanceFailed as i32)
                    }
                }
            }
        };

        WasmInstance { instance, store }
    }

    fn call_function<Params: wasmtime::WasmParams, Results: wasmtime::WasmResults>(
        &mut self,
        name: &'static str,
        params: Params,
    ) -> Result<Results, String> {
        let vertigo_entry_function = {
            self.instance
                .get_typed_func::<Params, Results>(&mut self.store, name)
                .map_err(|err| {
                    log::error!("Error calling function: {err}");
                    err.to_string()
                })?
        };

        vertigo_entry_function
            .call(&mut self.store, params)
            .map_err(|error| format!("{error}"))
    }

    pub fn call_vertigo_entry_function(&mut self) {
        self.call_function::<(u32, u32), ()>(
            "vertigo_entry_function",
            (super::VERTIGO_VERSION_MAJOR, super::VERTIGO_VERSION_MINOR),
        )
        .inspect_err(|err| log::error!("Error calling entry function: {err}"))
        .unwrap_or_default();
    }

    pub fn wasm_callback(&mut self, callback_id: u64, params: JsValue) -> JsValue {
        let mut data_context = DataContext::from_store(&mut self.store, self.instance);
        let params_ptr = data_context.save_value(params);

        let result = self
            .call_function::<(u64, u64), u64>(
                "vertigo_export_wasm_callback",
                (callback_id, params_ptr.get_long_ptr()),
            )
            .inspect_err(|err| log::error!("Error calling callback: {err}"))
            .unwrap_or_default();

        if result == 0 {
            JsValue::Undefined
        } else {
            //TODO - to implement
            todo!()
        }
    }

    pub fn handle_url(&mut self, url: &str) -> Option<ResponseState> {
        let url = JsValue::String(url.to_string());

        let params_ptr = {
            let mut data_context = DataContext::from_store(&mut self.store, self.instance);
            let params_ptr = data_context.save_value(url);
            params_ptr
        };

        let result = self
            .call_function::<u64, u64>("vertigo_export_handle_url", params_ptr.get_long_ptr())
            .inspect_err(|err| log::error!("Error calling callback: {err}"))
            .unwrap_or_default();

        let result = {
            let mut data_context = DataContext::from_store(&mut self.store, self.instance);
            let result = data_context.get_value_long_ptr(LongPtr::from(result));
            result
        };

        self.decode_response_state(result)
    }

    fn decode_response_state(&self, value: JsValue) -> Option<ResponseState> {
        if value == JsValue::Undefined {
            return None;
        }

        let JsValue::List(mut list) = value else {
            panic!("decode_response_state deecode error");
        };

        let Some(JsValue::Vec(body)) = list.pop() else {
            panic!("decode_response_state deecode error");
        };

        let Some(JsValue::Object(headers_value)) = list.pop() else {
            panic!("decode_response_state deecode error");
        };

        let Some(JsValue::U32(status)) = list.pop() else {
            panic!("decode_response_state deecode error");
        };

        if !list.is_empty() {
            panic!("decode_response_state deecode error");
        }

        let mut headers = HashMap::<String, String>::new();

        for (name, value) in headers_value {
            let JsValue::String(value) = value else {
                panic!("decode_response_state deecode error");
            };
            headers.insert(name, value);
        }
        let status = match StatusCode::from_u16(status as u16) {
            Ok(status) => status,
            Err(error) => {
                let error = format!("Incorrect status code: {}", error);
                return Some(ResponseState::internal_error(error));
            }
        };

        Some(ResponseState {
            status,
            headers,
            body,
        })
    }

    pub fn send_fetch_response(&mut self, callback_id: u64, response: FetchResponse) {
        let params = JsValue::List(vec![
            JsValue::bool(response.success),
            JsValue::U32(response.status),
            convert_body_to_value(response.body),
        ]);

        let result = self.wasm_callback(callback_id, params);
        assert_eq!(result, JsValue::Undefined);
    }
}
