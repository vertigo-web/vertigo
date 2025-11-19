use std::rc::Rc;

use vertigo_macro::store;

use crate::{
    command::{decode_json, CommandForWasm},
    driver_module::api::{api_fetch::api_fetch, api_location, api_timers, api_websocket},
    JsJson,
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
                CommandForWasm::TimerCall { callback } => {
                    api_timers().callback_timeout(callback);
                }
                CommandForWasm::LocationCall { callback, value } => {
                    api_location().callback(callback, value);
                }
            }
        }

        JsJson::Null
    }
}
