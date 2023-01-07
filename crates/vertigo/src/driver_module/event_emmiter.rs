use std::rc::Rc;
use crate::{struct_mut::{CounterMut, BTreeMapMut}, DropResource};

#[derive(Clone)]
pub struct EventEmmiter<T: Clone + 'static> {
    counter: Rc<CounterMut>,
    #[allow(clippy::type_complexity)]
    list: Rc<BTreeMapMut<u32, Rc<dyn Fn(T) + 'static>>>,
}

impl<T: Clone + 'static> Default for EventEmmiter<T> {
    fn default() -> Self {
        EventEmmiter {
            counter: Rc::new(CounterMut::new(1)),
            list: Rc::new(BTreeMapMut::new()),
        }
    }
}

impl<T: Clone + Send + Sync> EventEmmiter<T> {
    pub fn add<F: Fn(T) + 'static>(&self, callback: F) -> DropResource {
        let id = self.counter.get_next();

        self.list.insert(id, Rc::new(callback));

        DropResource::new({
            let list = self.list.clone();
            let id = id;
            move || {
                list.remove(&id);
            }
        })
    }

    pub fn trigger(&self, value: T) {
        let callback_list = self.list.map(|state| {
            state
                .iter()
                .map(|(_, callbcak)| {
                    callbcak.clone()
                })
                .collect::<Vec<_>>()
        });

        for callback in callback_list {
            callback(value.clone());
        }
    }
}
