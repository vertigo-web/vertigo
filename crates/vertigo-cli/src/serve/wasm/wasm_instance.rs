use std::{process::exit, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;
use vertigo::{
    JsJson, JsJsonSerialize,
    dev::{
        CallbackId, LongPtr, SsrFetchResponse,
        command::{CommandForBrowser, CommandForWasm, decode_json},
    },
};
use wasmtime::{Caller, Engine, Func, Instance, Module, Store};

use crate::{
    commons::ErrorCode,
    serve::{
        request_state::RequestState,
        response_state::ResponseState,
        timings::{ENTRY_FUNCTION, HANDLE_URL_FUNCTION, SsrProbe, WASM_COMMAND_FUNCTION},
    },
};

use super::{data_context::DataContext, message::Message};

pub struct WasmInstance {
    instance: Instance,
    store: Store<RequestState>,
    probe: SsrProbe,
    /// The import-order workaround below was needed. Reported through
    /// [`WasmInstance::instantiate_retried`] so a doubled instantiation time explains
    /// itself rather than reading as noise.
    retried: bool,
}

impl WasmInstance {
    pub fn new(
        sender: UnboundedSender<Message>,
        engine: &Engine,
        module: &Module,
        request: RequestState,
        probe: SsrProbe,
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
            let probe = probe.clone();

            Func::wrap(
                &mut store,
                move |caller: Caller<'_, RequestState>, long_ptr: u64| -> u64 {
                    // The whole body, not just the dispatch: pulling the argument out of
                    // linear memory and writing the answer back are host work proportional
                    // to the batch size, and a `DomBulkUpdate` blob is the largest thing
                    // that ever crosses here. Timing only the dispatch would charge that
                    // copying to wasm execution - exactly the mis-attribution the
                    // benchmark exists to avoid.
                    let host_mark = probe.start();

                    let long_ptr = LongPtr::from(long_ptr);
                    let mut data_context = DataContext::from_caller(caller);

                    let value = data_context.get_value_long_ptr(long_ptr);

                    let result = decode_json::<CommandForBrowser>(value)
                        .map(|item| handle_command(request.clone(), item));

                    let result = match result {
                        Ok(result) => data_context.save_value(result).get_long_ptr(),
                        Err(err) => {
                            log::error!("import_dom_access -> decode error = {err}");
                            0
                        }
                    };

                    probe.host_call(host_mark);
                    result
                },
            )
        };

        let mut imports = [import_dom_access.into(), import_panic_message.into()];
        let mut retried = false;
        let instance = match Instance::new(&mut store, module, &imports) {
            Ok(instance) => instance,
            Err(err) => {
                // Workaround for rust/wasmtime mangling with functions order.
                // Upon error try with panic/dom_access reversed before giving up.
                imports.reverse();
                retried = true;
                match Instance::new(&mut store, module, &imports) {
                    Ok(instance) => {
                        log::warn!(
                            "WASM instantiation types order problem - update rust or soon it will stop working"
                        );
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

        WasmInstance {
            instance,
            store,
            probe,
            retried,
        }
    }

    pub fn instantiate_retried(&self) -> bool {
        self.retried
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

        // Every call into wasm passes through here - the mount, `handle_url`, and the
        // timer and fetch-response re-entries from the drain loop - so this is the one
        // place phase 2 has to be measured. Deliberately outside the `get_typed_func`
        // lookup above: that is a host-side export-map lookup repeated on every call, and
        // charging it to wasm would hide it. It lands in `unaccounted` instead.
        let call_mark = self.probe.start();

        let result = vertigo_entry_function
            .call(&mut self.store, params)
            .map_err(|error| format!("{error}"));

        self.probe.wasm_call(name, call_mark);
        result
    }

    pub fn call_vertigo_entry_function(&mut self) {
        self.call_function::<(u32, u32), ()>(
            ENTRY_FUNCTION,
            (super::VERTIGO_VERSION_MAJOR, super::VERTIGO_VERSION_MINOR),
        )
        .inspect_err(|err| log::error!("Error calling entry function: {err}"))
        .unwrap_or_default();
    }

    pub fn wasm_command(&mut self, command: CommandForWasm) -> JsJson {
        let mut data_context = DataContext::from_store(&mut self.store, self.instance);
        let params_ptr = data_context.save_value(command.to_json());

        let _result = self
            .call_function::<u64, u64>(WASM_COMMAND_FUNCTION, params_ptr.get_long_ptr())
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
            .call_function::<u64, u64>(HANDLE_URL_FUNCTION, params_ptr.get_long_ptr())
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
