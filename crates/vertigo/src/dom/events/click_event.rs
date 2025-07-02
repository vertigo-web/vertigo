use std::rc::Rc;

use crate::{struct_mut::ValueMut, JsValue};

/// Structure passed as a parameter to callback on on_key_down event.
#[derive(Clone, Debug, Default)]
pub struct ClickEvent {
    inner: Rc<ValueMut<ClickEventInner>>,
}

#[derive(Clone, Debug, Default)]
pub struct ClickEventInner {
    stop_propagation: bool,
    prevent_default: bool,
}

impl ClickEvent {
    pub fn stop_propagation(&self) {
        self.inner.change(|inner| inner.stop_propagation = true);
    }

    pub fn prevent_default(&self) {
        self.inner.change(|inner| inner.prevent_default = true);
    }
}

impl From<ClickEvent> for JsValue {
    fn from(val: ClickEvent) -> JsValue {
        let inner = val.inner.get();
        JsValue::Object([
            ("stop_propagation".to_string(), JsValue::from(inner.stop_propagation)),
            ("prevent_default".to_string(), JsValue::from(inner.prevent_default)),
        ].into_iter().collect())
    }
}
