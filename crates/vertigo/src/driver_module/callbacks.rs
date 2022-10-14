use std::rc::Rc;

use crate::{struct_mut::HashMapMut, JsValue, DropResource};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct CallbackId(u64);

fn get_unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

impl CallbackId {
    #[allow(clippy::new_without_default)]
    pub fn new() -> CallbackId {
        CallbackId(get_unique_id())
    }

    pub fn to_u64(&self) -> u64 {
        self.0
    }

    pub fn from_u64(id: u64) -> Self {
        Self(id)
    }
}

type CallBackFn = dyn Fn(JsValue) -> JsValue + 'static;

pub struct CallbackStore {
    data: Rc<HashMapMut<CallbackId, Rc<CallBackFn>>>,
}

impl CallbackStore {
    pub fn new() -> CallbackStore {
        CallbackStore {
            data: Rc::new(HashMapMut::new()),
        }
    }

    pub fn register<C: Fn(JsValue) -> JsValue + 'static>(&self, callback: C) -> (CallbackId, DropResource) {
        let callback = Rc::new(callback);
        let id = CallbackId::new();

        self.data.insert(id.clone(), callback);

        let drop = DropResource::new({
            let id = id.clone();
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
            Some(callback) => {
                callback(value)
            },
            None => {
                log::error!("callback id not found = {callback_id:?}");
                JsValue::Undefined
            }
        }
    }
}
