use std::rc::Rc;

use vertigo_macro::store;

use crate::{
    dev::command::{decode_json, CommandForWasm},
    JsJson,
};

use super::{api_fetch::api_fetch, api_location, api_timers, api_websocket};

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
                CommandForWasm::CallbackCall { callback_id, value } => {
                    use crate::driver_module::api::callbacks::api_callbacks;
                    return api_callbacks().call(callback_id, value);
                }
            }
        }

        JsJson::Null
    }
}
