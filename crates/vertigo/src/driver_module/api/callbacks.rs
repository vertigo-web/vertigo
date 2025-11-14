use std::rc::Rc;

use crate::{dev::CallbackId, struct_mut::HashMapMut, DropResource, JsValue};

type CallBackFn = dyn Fn(JsValue) -> JsValue + 'static;

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

    pub fn register<C: Fn(JsValue) -> JsValue + 'static>(
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

    pub fn register_with_id<C: Fn(CallbackId, JsValue) -> JsValue + 'static>(
        &self,
        callback: C,
    ) -> (CallbackId, DropResource) {
        let id = CallbackId::new();
        let callback = Rc::new(callback);

        self.data
            .insert(id, Rc::new(move |data| callback(id, data)));

        let drop = DropResource::new({
            let data = self.data.clone();
            move || {
                data.remove(&id);
            }
        });

        (id, drop)
    }

    pub fn call(&self, callback_id: CallbackId, value: JsValue) -> JsValue {
        let callback = self.data.get(&callback_id);

        match callback {
            Some(callback) => callback(value),
            None => {
                log::error!("callback id not found = {callback_id:?}");
                JsValue::Undefined
            }
        }
    }
}

use vertigo_macro::store;

#[store]
pub fn api_callbacks() -> CallbackStore {
    CallbackStore::new()
}
