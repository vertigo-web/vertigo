use std::{
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};
use vertigo_macro::store;

use crate::{JsJson, JsJsonContext, JsJsonDeserialize, JsJsonNumber, JsJsonSerialize};

#[derive(PartialEq, Eq, Hash, Clone, Debug, Copy)]
pub struct CallbackId(u64);

#[store]
pub fn get_counter() -> Rc<AtomicU64> {
    Rc::new(AtomicU64::new(1))
}

impl CallbackId {
    #[allow(clippy::new_without_default)]
    pub fn new() -> CallbackId {
        CallbackId(get_counter().fetch_add(1, Ordering::Relaxed))
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn from_u64(id: u64) -> Self {
        Self(id)
    }
}

impl JsJsonSerialize for CallbackId {
    fn to_json(self) -> JsJson {
        JsJson::Number(JsJsonNumber(self.0 as f64))
    }
}

impl JsJsonDeserialize for CallbackId {
    fn from_json(_context: JsJsonContext, json: JsJson) -> Result<Self, JsJsonContext> {
        if let JsJson::Number(JsJsonNumber(value)) = json {
            return Ok(CallbackId::from_u64(value as u64));
        }

        Err(JsJsonContext::new(format!(
            "Expected Number, received={}",
            json.typename()
        )))
    }
}
