use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{
    struct_mut::{HashMapMut, ValueMut},
    DropResource, JsValue,
};

#[derive(PartialEq, Eq, Hash, Clone, Debug, Copy)]
pub struct CallbackId(u64);

static COUNTER: AtomicU64 = AtomicU64::new(1);

impl CallbackId {
    #[allow(clippy::new_without_default)]
    pub fn new() -> CallbackId {
        CallbackId(
            COUNTER.fetch_add(1, Ordering::Relaxed)
        )
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn from_u64(id: u64) -> Self {
        Self(id)
    }

    #[cfg(test)]
    pub fn reset() {
        COUNTER.store(1, Ordering::Relaxed)
    }
}

type CallBackFn = dyn Fn(JsValue) -> JsValue + 'static;

#[derive(Clone)]
pub struct CallbackStore {
    data: Rc<HashMapMut<CallbackId, Rc<CallBackFn>>>,
}

impl CallbackStore {
    pub fn new() -> CallbackStore {
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

    pub fn register_once<C: Fn(JsValue) -> JsValue + 'static>(&self, callback: C) -> CallbackId {
        let drop_value = Rc::new(ValueMut::new(None));

        let (callback_id, drop) = self.register({
            let drop_value = drop_value.clone();

            move |node| {
                let result = callback(node);
                drop_value.set(None);
                result
            }
        });

        drop_value.set(Some(drop));

        callback_id
    }
}
