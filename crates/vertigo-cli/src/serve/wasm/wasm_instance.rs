use std::{process::exit, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;
use vertigo::command::{decode_json, CommandForBrowser, CommandForWasm};
use vertigo::{CallbackId, JsJson, JsJsonSerialize, LongPtr, SsrFetchResponse};
use wasmtime::{Caller, Engine, Func, Instance, Module, Store};

use crate::{
    commons::ErrorCode,
    serve::{request_state::RequestState, response_state::ResponseState},
};

use super::{data_context::DataContext, message::Message};

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
        handle_command: Arc<
            dyn Fn(RequestState, CommandForBrowser) -> JsJson + 'static + Send + Sync,
        >,
    ) -> Self {
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

                    let result = decode_json::<CommandForBrowser>(value)
                        .map(|item| handle_command(request.clone(), item));

                    match result {
                        Ok(result) => data_context.save_value(result).get_long_ptr(),
                        Err(err) => {
                            log::error!("import_dom_access -> decode error = {err}");
                            0
                        }
                    }
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

    pub fn wasm_command(&mut self, command: CommandForWasm) -> JsJson {
        let mut data_context = DataContext::from_store(&mut self.store, self.instance);
        let params_ptr = data_context.save_value(command.to_json());

        let _result = self
            .call_function::<u64, u64>("vertigo_export_wasm_command", params_ptr.get_long_ptr())
            .inspect_err(|err| log::error!("Error calling callback: {err}"))
            .unwrap_or_default();

        JsJson::Null
    }

    pub fn handle_url(&mut self, url: &str) -> Option<ResponseState> {
        let url = JsJson::String(url.to_string());

        let params_ptr = {
            let mut data_context = DataContext::from_store(&mut self.store, self.instance);
            data_context.save_value(url)
        };

        let result = self
            .call_function::<u64, u64>("vertigo_export_handle_url", params_ptr.get_long_ptr())
            .inspect_err(|err| log::error!("Error calling callback: {err}"))
            .unwrap_or_default();

        let result = {
            let mut data_context = DataContext::from_store(&mut self.store, self.instance);
            data_context.get_value_long_ptr(LongPtr::from(result))
        };

        self.decode_response_state(result)
    }

    fn decode_response_state(&self, value: JsJson) -> Option<ResponseState> {
        if let JsJson::Null = value {
            return None;
        }

        let response: Result<ResponseState, vertigo::JsJsonContext> =
            decode_json::<ResponseState>(value);

        if let Ok(response) = response {
            return Some(response);
        }

        log::error!("decode_response_state: decode error = {response:#?}");

        None
    }

    pub fn send_fetch_response(&mut self, callback: CallbackId, response: SsrFetchResponse) {
        let result = self.wasm_command(CommandForWasm::FetchExecResponse { response, callback });
        assert_eq!(result, JsJson::Null);
    }
}
