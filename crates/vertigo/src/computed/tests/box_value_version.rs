use crate::{computed::Computed, struct_mut::ValueMut};
use std::rc::Rc;

struct SubscribeValueVerInner<T: Clone> {
    version: ValueMut<u32>,
    value: ValueMut<Option<T>>,
}

impl<T: Clone> SubscribeValueVerInner<T> {
    pub fn new() -> Rc<SubscribeValueVerInner<T>> {
        Rc::new(
            SubscribeValueVerInner {
                version: ValueMut::new(0),
                value: ValueMut::new(None),
            }
        )
    }
}

use crate::computed::Client;

pub struct SubscribeValueVer<T: PartialEq + Clone> {
    _client: Option<Client>,
    value: Rc<SubscribeValueVerInner<T>>,
}

impl<T: PartialEq + Clone> SubscribeValueVer<T> {
    pub fn new(com: Computed<T>) -> SubscribeValueVer<T> {
        let value = SubscribeValueVerInner::new();

        let client = {
            let value = value.clone();
            com.subscribe(move |new_value| {
                value.value.set(Some(new_value.clone()));
                let current = value.version.get();
                value.version.set(current + 1);
            })
        };

        SubscribeValueVer {
            _client: Some(client),
            value,
        }
    }

    pub fn get(&self) -> (T, u32) {
        let value = self.value.value.get().unwrap();
        let version = self.value.version.get();

        (value, version)
    }

    pub fn off(&mut self) {
        self._client = None;
    }
}
