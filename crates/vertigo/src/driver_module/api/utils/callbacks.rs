use std::rc::Rc;

use crate::{
    dev::CallbackId,
    struct_mut::{HashMapMut, ValueMut},
    DropResource,
};

type CallBackFn<R, R2> = dyn Fn(R) -> R2;

#[derive(Clone)]
pub struct CallbackStore<R, R2: Default> {
    data: Rc<HashMapMut<CallbackId, Rc<CallBackFn<R, R2>>>>,
}

impl<R: 'static, R2: Default + 'static> CallbackStore<R, R2> {
    pub fn new() -> CallbackStore<R, R2> {
        CallbackStore {
            data: Rc::new(HashMapMut::new()),
        }
    }

    pub fn register(&self, callback: impl Fn(R) -> R2 + 'static) -> (CallbackId, DropResource) {
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

    pub fn register_with_id(&self, callback: impl Fn(CallbackId, R) -> R2 + 'static) -> (CallbackId, DropResource) {
        let callback = Rc::new(callback);
        let id = CallbackId::new();

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

    pub fn call(&self, callback_id: CallbackId, value: R) -> R2 {
        let callback = self.data.get(&callback_id);

        match callback {
            Some(callback) => callback(value),
            None => {
                log::error!("callback id not found = {callback_id:?}");
                R2::default()
            }
        }
    }

    pub fn register_once(&self, callback: impl Fn(R) -> R2 + 'static) -> CallbackId {
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
