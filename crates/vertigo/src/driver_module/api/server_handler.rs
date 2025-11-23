use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

use vertigo_macro::store;

use crate::{struct_mut::ValueMut, JsJson, JsJsonNumber};

type PlainHandler = dyn Fn(&str) -> Option<String>;

fn convert_to_jsvalue(status: u32, headers: HashMap<String, String>, body: Vec<u8>) -> JsJson {
    let headers = headers
        .into_iter()
        .map(|(key, value)| (key, JsJson::String(value)))
        .collect::<BTreeMap<_, _>>();

    JsJson::List(vec![
        JsJson::Number(JsJsonNumber(status as f64)),
        JsJson::Object(headers),
        JsJson::List(
            body.into_iter()
                .map(|byte| JsJson::Number(JsJsonNumber(byte as f64)))
                .collect(),
        ),
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

    pub fn handler(&self, url: &str) -> JsJson {
        self.plains_handler.map(|handler| match handler {
            Some(handler) => {
                if let Some(response) = handler(url).map(|inner| inner.into_bytes()) {
                    let mut headers = HashMap::<String, String>::new();
                    headers.insert("content-type".into(), "text/plain".into());

                    return convert_to_jsvalue(200, headers, response);
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
