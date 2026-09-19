use std::process::exit;
use vertigo::{
    JsJson, JsJsonSerialize,
    dev::{
        CallbackId, LongPtr, SsrFetchResponse,
        command::{CommandForBrowser, CommandForWasm, decode_json},
    },
};
use wasmtime::{Caller, Engine, Instance, InstancePre, Linker, Store};

use crate::{
    commons::ErrorCode,
    serve::{
        response_state::ResponseState,
        timings::{ENTRY_FUNCTION, HANDLE_URL_FUNCTION, SsrProbe, WASM_COMMAND_FUNCTION},
    },
};

use super::{data_context::DataContext, host_state::HostState, message::Message};

/// The module the guest imports from - `#[link(wasm_import_module = "mod")]` in
/// `vertigo::external_api`, and the `mod` key of the import object the browser builds in
/// `wasm_module.ts`.
const IMPORT_MODULE: &str = "mod";

/// Defines the host functions the guest imports, keyed by name.
///
/// The closures capture nothing, reaching their per-request collaborators through
/// [`HostState`] on the `Store`. That is what allows one linker, built once at startup, to
/// serve every request.
pub fn build_linker(engine: &Engine) -> Result<Linker<HostState>, ErrorCode> {
    fn registration_failed(name: &str, err: impl std::fmt::Debug) -> ErrorCode {
        log::error!("WASM host function registration failed: {IMPORT_MODULE}.{name}: {err:?}");
        ErrorCode::ServeWasmInstanceFailed
    }

    let mut linker = Linker::new(engine);

    linker
        .func_wrap(
            IMPORT_MODULE,
            "panic_message",
            |caller: Caller<'_, HostState>, long_ptr: u64| {
                // `Caller::data` borrows and `DataContext::from_caller` moves, so whatever
                // the body needs is taken off the state first.
                let sender = caller.data().sender.clone();

                let mut data_context = DataContext::from_caller(caller);
                let long_ptr = LongPtr::from(long_ptr);
                let (ptr, offset) = long_ptr.into_parts();

                let message = data_context.get_string_from(ptr, offset);
                log::error!("wasm panic: {message:?}");

                sender.send(Message::Panic(message)).unwrap_or_default();
            },
        )
        .map_err(|err| registration_failed("panic_message", err))?;

    linker
        .func_wrap(
            IMPORT_MODULE,
            "dom_access",
            |caller: Caller<'_, HostState>, long_ptr: u64| -> u64 {
                let state = caller.data();
                let probe = state.probe.clone();
                let request = state.request.clone();
                let handle_command = state.handle_command.clone();

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
                    .map(|item| handle_command(request, item));

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
        .map_err(|err| registration_failed("dom_access", err))?;

    Ok(linker)
}

pub struct WasmInstance {
    instance: Instance,
    store: Store<HostState>,
    probe: SsrProbe,
}

impl WasmInstance {
    /// Instantiates from an [`InstancePre`] whose imports were resolved at startup.
    pub fn new(engine: &Engine, instance_pre: &InstancePre<HostState>, state: HostState) -> Self {
        let probe = state.probe.clone();
        let mut store = Store::new(engine, state);

        // A failure here is a trap in the start function or an allocation failure.
        let instance = match instance_pre.instantiate(&mut store) {
            Ok(instance) => instance,
            Err(err) => {
                log::error!("WASM instantiation error: {err:?}");
                exit(ErrorCode::ServeWasmInstanceFailed as i32)
            }
        };

        WasmInstance {
            instance,
            store,
            probe,
        }
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

#[cfg(test)]
mod tests {
    use wasmtime::Module;

    use super::*;

    /// The two imports as the guest declares them in `vertigo::external_api`.
    const PANIC_FIRST: &str = r#"(module
        (import "mod" "panic_message" (func (param i64)))
        (import "mod" "dom_access"    (func (param i64) (result i64))))"#;

    /// The same module with the import section emitted the other way round. `wasm-ld` picks
    /// the order, and it differs between build profiles - `lto`/`codegen-units` change it -
    /// as well as across toolchains. Both must resolve.
    const DOM_ACCESS_FIRST: &str = r#"(module
        (import "mod" "dom_access"    (func (param i64) (result i64)))
        (import "mod" "panic_message" (func (param i64))))"#;

    /// Whether a module's imports resolve against the real linker. The module is parsed in a
    /// separate step so a typo in the WAT above cannot pass for a resolution failure.
    fn imports_resolve(wat: &str) -> bool {
        let engine = Engine::default();

        let Ok(module) = Module::new(&engine, wat) else {
            panic!("test WAT does not parse");
        };

        let Ok(linker) = build_linker(&engine) else {
            panic!("build_linker failed");
        };

        linker.instantiate_pre(&module).is_ok()
    }

    #[test]
    fn imports_resolve_in_either_declared_order() {
        assert!(imports_resolve(PANIC_FIRST));
        assert!(imports_resolve(DOM_ACCESS_FIRST));
    }

    #[test]
    fn an_unknown_import_does_not_resolve() {
        // Guards the assertion above against passing for the wrong reason: if the linker
        // matched by position it would happily satisfy this too, since the signatures and
        // the arity are those of the real pair.
        let wat = r#"(module
            (import "mod" "dom_access_typo" (func (param i64) (result i64)))
            (import "mod" "panic_message"   (func (param i64))))"#;

        assert!(!imports_resolve(wat));
    }
}
