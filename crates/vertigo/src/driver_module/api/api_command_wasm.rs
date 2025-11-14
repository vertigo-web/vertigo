use std::rc::Rc;

use vertigo_macro::store;

use crate::{JsJson, command::{CommandWasm, decode_json}, driver_module::api::api_fetch::api_fetch};

#[store]
pub fn api_command_wasm() -> Rc<ApiCommandWasm> {

    Rc::new(ApiCommandWasm {})
}

pub struct ApiCommandWasm {}

impl ApiCommandWasm {

    pub fn command_from_js(self: &Rc<ApiCommandWasm>, json: JsJson) -> JsJson {

        let response = decode_json::<CommandWasm>(json);

        match response {
            CommandWasm::FetchExecResponse { response, callback } => {
                api_fetch().callbak(callback, response);
                JsJson::Null
            }
        }
    }
}

