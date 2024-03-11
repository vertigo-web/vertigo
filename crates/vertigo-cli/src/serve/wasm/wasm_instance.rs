use tokio::sync::mpsc::UnboundedSender;
use wasmtime::{Caller, Engine, Func, Instance, Module, Store};

use crate::serve::{js_value::JsValue, request_state::RequestState};

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

            move |caller: Caller<'_, RequestState>, ptr: u32, offset: u32| {
                let mut data_context = DataContext::from_caller(caller);

                let message = data_context.get_string_from(ptr, offset);
                log::error!("panic: {message:?}");

                sender.send(Message::Panic(message)).unwrap();
            }
        });

        let import_dom_access = {
            Func::wrap(
                &mut store,
                move |caller: Caller<'_, RequestState>, ptr: u32, offset: u32| -> u32 {
                    let mut data_context = DataContext::from_caller(caller);

                    let value = data_context.get_value(ptr, offset);

                    // Ignore cookie operations
                    if let Ok(()) = match_cookie_command(&value) {
                        return 0;
                    }

                    // Intercept plain response
                    if let Ok(body) = match_plain_response(&value) {
                        sender.send(Message::PlainResponse(body)).unwrap();
                        return 0;
                    }

                    //get history router location
                    if let Ok(()) = match_history_router(&value) {
                        let result = JsValue::str(url.clone());
                        return data_context.save_value(result);
                    }

                    if let Ok(env_name) = match_get_env(&value) {
                        let env_value = request.env(env_name);

                        let result = match env_value {
                            Some(value) => JsValue::String(value),
                            None => JsValue::Null,
                        };
                        return data_context.save_value(result);
                    }

                    //adding callback for hashrouter
                    if match_history_router_callback(&value).is_ok() {
                        return 0;
                    }

                    if let Ok(data) = match_dom_bulk_update(&value) {
                        sender.send(Message::DomUpdate(data)).unwrap();
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
                        return data_context.save_value(current_time);
                    }

                    if let Ok(result) = match_interval(&value) {
                        match result {
                            CallWebsocketResult::TimeoutSet { time, callback_id } => {
                                if time == 0 {
                                    sender
                                        .send(Message::SetTimeoutZero { callback_id })
                                        .unwrap();
                                }

                                let result = JsValue::I32(0); // fake timerId
                                return data_context.save_value(result);
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
                            .unwrap();
                        return 0;
                    }

                    if let Ok(()) = match_is_browser(&value) {
                        let result = JsValue::bool(false);
                        return data_context.save_value(result);
                    }

                    log::error!("unsupported message: {value:#?}");
                    0
                },
            )
        };

        let imports = [import_panic_message.into(), import_dom_access.into()];
        let instance = Instance::new(&mut store, module, &imports).unwrap();

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
                .unwrap()
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
        .unwrap();
    }

    pub fn wasm_callback(&mut self, callback_id: u64, params: JsValue) -> JsValue {
        let mut data_context = DataContext::from_store(&mut self.store, self.instance);
        let params_ptr = data_context.save_value(params);

        let result = self
            .call_function::<(u64, u32), u64>("wasm_callback", (callback_id, params_ptr))
            .unwrap();

        if result == 0 {
            JsValue::Undefined
        } else {
            //TODO - to implement
            todo!()
        }
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
