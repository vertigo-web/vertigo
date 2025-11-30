use std::{collections::HashMap, rc::Rc};
use vertigo::AutoJsJson;
use vertigo_macro::store;

use crate::{computed::struct_mut::ValueMut, driver_module::js_value::JsJsonSerialize, JsJson};

type PlainHandler = dyn Fn(&str) -> Option<String>;

#[derive(AutoJsJson)]
pub struct ResponseState {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

pub struct ServerHandler {
    plains_handler: ValueMut<Option<Rc<PlainHandler>>>,
}

impl ServerHandler {
    pub fn new() -> ServerHandler {
        ServerHandler {
            plains_handler: ValueMut::new(None),
        }
    }

    pub fn plains(&self, callback: impl Fn(&str) -> Option<String> + 'static) {
        self.plains_handler.set(Some(Rc::new(callback)));
    }

    pub fn handler(&self, url: &str) -> JsJson {
        self.plains_handler.map(|handler| match handler {
            Some(handler) => {
                if let Some(response) = handler(url).map(|inner| inner.into_bytes()) {
                    let mut headers = HashMap::<String, String>::new();
                    headers.insert("content-type".into(), "text/plain".into());

                    return ResponseState {
                        status: 200,
                        headers,
                        body: response,
                    }
                    .to_json();
                }

                JsJson::Null
            }
            None => JsJson::Null,
        })
    }
}

#[store]
pub fn api_server_handler() -> Rc<ServerHandler> {
    Rc::new(ServerHandler::new())
}
