use std::rc::Rc;

use vertigo_macro::store;

use crate::{
    JsJson, command::{CommandForWasm, decode_json}, driver_module::api::{api_fetch::api_fetch, api_websocket}
};

#[store]
pub fn api_command_wasm() -> Rc<CommandWasmApi> {
    Rc::new(CommandWasmApi {})
}

pub struct CommandWasmApi {}

impl CommandWasmApi {
    pub fn command_from_js(self: &Rc<CommandWasmApi>, json: JsJson) -> JsJson {
        let command = decode_json::<CommandForWasm>(json)
            .inspect_err(|err| {
                log::error!("command_from_js -> decode error = {err}");
            })
            .ok();

        if let Some(command) = command {
            match command {
                CommandForWasm::FetchExecResponse { response, callback } => {
                    api_fetch().callback(callback, response);
                }
                CommandForWasm::Websocket { callback, message } => {
                    api_websocket().callback(callback, message);
                }
            }
        }

        JsJson::Null
    }
}
