use std::{collections::HashMap, rc::Rc};

use vertigo_macro::store;

use crate::{struct_mut::ValueMut, JsValue};

type PlainHandler = dyn Fn(&str) -> Option<String>;

fn convert_to_jsvalue(status: u32, headers: HashMap<String, String>, body: Vec<u8>) -> JsValue {
    let headers = headers
        .into_iter()
        .map(|(key, value)| (key, JsValue::String(value)))
        .collect::<HashMap<_, _>>();

    JsValue::List(vec![
        JsValue::U32(status),
        JsValue::Object(headers),
        JsValue::Vec(body),
    ])
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

    pub fn handler(&self, url: &str) -> JsValue {
        self.plains_handler.map(|handler| match handler {
            Some(handler) => {
                if let Some(response) = handler(url).map(|inner| inner.into_bytes()) {
                    let mut headers = HashMap::<String, String>::new();
                    headers.insert("content-type".into(), "text/plain".into());

                    return convert_to_jsvalue(200, headers, response);
                }

                JsValue::Undefined
            }
            None => JsValue::Undefined,
        })
    }
}

#[store]
pub fn api_server_handler() -> Rc<ServerHandler> {
    Rc::new(ServerHandler::new())
}
