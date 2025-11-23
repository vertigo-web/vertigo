use std::rc::Rc;

use crate::{dev::CallbackId, struct_mut::HashMapMut, DropResource, JsJson};

type CallBackFn = dyn Fn(JsJson) -> JsJson + 'static;

#[derive(Clone)]
pub struct CallbackStore {
    data: Rc<HashMapMut<CallbackId, Rc<CallBackFn>>>,
}

impl CallbackStore {
    fn new() -> CallbackStore {
        CallbackStore {
            data: Rc::new(HashMapMut::new()),
        }
    }

    pub fn register<C: Fn(JsJson) -> JsJson + 'static>(
        &self,
        callback: C,
    ) -> (CallbackId, DropResource) {
        let callback = Rc::new(callback);
        let id = CallbackId::new();

        self.data.insert(id, callback);

        let drop = DropResource::new({
            let data = self.data.clone();
            move || {
                data.remove(&id);
            }
        });

        (id, drop)
    }

    pub fn call(&self, callback_id: CallbackId, value: JsJson) -> JsJson {
        let callback = self.data.get(&callback_id);

        match callback {
            Some(callback) => callback(value),
            None => {
                log::error!("callback id not found = {callback_id:?}");
                JsJson::Null
            }
        }
    }
}

use vertigo_macro::store;

#[store]
pub fn api_callbacks() -> CallbackStore {
    CallbackStore::new()
}
