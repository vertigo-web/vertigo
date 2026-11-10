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
    fn from_json(context: JsJsonContext, json: JsJson) -> Result<Self, JsJsonContext> {
        u64::from_json(context, json).map(CallbackId::from_u64)
    }
}

#[cfg(test)]
mod tests {
    use super::CallbackId;
    use crate::{JsJson, JsJsonNumber, from_json};

    #[test]
    fn id_is_a_whole_positive_number() {
        let number = |value: f64| JsJson::Number(JsJsonNumber(value));

        assert_eq!(from_json::<CallbackId>(number(7.0)), Ok(CallbackId(7)));
        for value in [-1.0, 1.5, f64::NAN] {
            assert!(from_json::<CallbackId>(number(value)).is_err(), "{value}");
        }
    }
}
