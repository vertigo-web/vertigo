use alloc::rc::Rc;
use crate::computed::Computed;

struct SubscribeValueVerInner<T> {
    version: u32,
    value: Option<T>
}

impl<T> SubscribeValueVerInner<T> {
    pub fn new() -> Rc<BoxRefCell<SubscribeValueVerInner<T>>> {
        Rc::new(
            BoxRefCell::new(
                SubscribeValueVerInner {
                    version: 0,
                    value: None,
                }
            )
        )
    }
}

use crate::computed::Client;
use crate::utils::BoxRefCell;

pub struct SubscribeValueVer<T: PartialEq + Clone> {
    client: Option<Client>,
    value: Rc<BoxRefCell<SubscribeValueVerInner<T>>>,
}

impl<T: PartialEq + Clone> SubscribeValueVer<T> {
    pub fn new(com: Computed<T>) -> SubscribeValueVer<T> {
        let value = SubscribeValueVerInner::new();

        let client = {
            let value = value.clone();
            com.subscribe(move |new_value| {
                value.change(new_value, |state, new_value| {
                    state.value = Some(new_value.clone());
                    state.version += 1;
                });
            })
        };

        SubscribeValueVer {
            client: Some(client),
            value
        }
    }

    pub fn get(&self) -> (T, u32) {
        self.value.get(|state| {
            if let Some(value) = &state.value {
                let version = state.version;

                (value.clone(), version)
            } else {
                panic!("expected value");
            }
        })
    }

    pub fn off(&mut self) {
        self.client = None;
    }
}
