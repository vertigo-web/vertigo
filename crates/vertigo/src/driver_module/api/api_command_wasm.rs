use std::rc::Rc;

use vertigo_macro::store;

use crate::{
    command::{decode_json, CommandForWasm},
    driver_module::api::api_fetch::api_fetch,
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
                    api_fetch().callbak(callback, response);
                }
            }
        }

        JsJson::Null
    }
}
