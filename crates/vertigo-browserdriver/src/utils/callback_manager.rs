use std::rc::Rc;

use super::{counter_rc::CounterRc, hash_map_rc::HashMapRc};

#[derive(Clone)]
pub struct CallbackManager<V> {
    next_id: CounterRc,
    data: HashMapRc<u64, Rc<dyn Fn(&V)>>,
}

impl<V> CallbackManager<V> {
    pub fn new() -> CallbackManager<V> {
        CallbackManager {
            next_id: CounterRc::new(1),
            data: HashMapRc::new("CallbackManager"),
        }
    }

    pub fn set<F: Fn(&V) + 'static>(&self, callback: F) -> u64 {
        let next_id = self.next_id.get_next();
        self.data.insert(next_id, Rc::new(callback));
        next_id
    }

    pub fn get(&self, callback_id: u64) -> Option<Rc<dyn Fn(&V)>> {
        self.data.must_get_clone(&callback_id)
    }

    pub fn remove(&self, callback_id: u64) {
        self.data.remove(&callback_id);
    }

    pub fn trigger(&self, value: V) {
        let callback_list = self.data.get_all_values();

        for callback in callback_list.into_iter() {
            callback(&value);
        }
    }
}


#[derive(Clone)]
pub struct CallbackManagerOwner<V> {
    next_id: CounterRc,
    data: HashMapRc<u64, Rc<dyn Fn(V)>>,
}

impl<V> CallbackManagerOwner<V> {
    pub fn new() -> CallbackManagerOwner<V> {
        CallbackManagerOwner {
            next_id: CounterRc::new(1),
            data: HashMapRc::new("CallbackManager"),
        }
    }

    pub fn set(&self, callback: impl Fn(V) + 'static) -> u64 {
        let next_id = self.next_id.get_next();
        self.data.insert(next_id, Rc::new(callback));
        next_id
    }

    fn get(&self, callback_id: u64) -> Option<Rc<dyn Fn(V)>> {
        self.data.must_get_clone(&callback_id)
    }

    pub fn remove(&self, callback_id: u64) {
        self.data.remove(&callback_id);
    }

    pub fn trigger(&self, callback_id: u64, value: V) {
        let callback = self.get(callback_id);

        match callback {
            Some(callback) => {
                callback(value);
            },
            None => {
                log::error!("Missing callback id {} ", callback_id);
            }
        }
    }
}