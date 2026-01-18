use std::rc::Rc;

use crate::computed::{
    DropResource,
    struct_mut::{BTreeMapMut, CounterMut},
};

#[derive(Clone)]
pub struct EventEmitter<T: Clone + 'static> {
    counter: Rc<CounterMut>,
    #[allow(clippy::type_complexity)]
    list: Rc<BTreeMapMut<u32, Rc<dyn Fn(T) + 'static>>>,
}

impl<T: Clone + 'static> Default for EventEmitter<T> {
    fn default() -> Self {
        EventEmitter {
            counter: Rc::new(CounterMut::new(1)),
            list: Rc::new(BTreeMapMut::new()),
        }
    }
}

impl<T: Clone + Send + Sync> EventEmitter<T> {
    pub fn add<F: Fn(T) + 'static>(&self, callback: F) -> DropResource {
        let id = self.counter.get_next();

        self.list.insert(id, Rc::new(callback));

        DropResource::new({
            let list = self.list.clone();
            move || {
                list.remove(&id);
            }
        })
    }

    pub fn trigger(&self, value: &T) {
        let callback_list = self
            .list
            .map(|state| state.values().cloned().collect::<Vec<_>>());

        for callback in callback_list {
            callback(value.clone());
        }
    }
}
