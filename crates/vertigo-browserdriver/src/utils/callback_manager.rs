use std::rc::Rc;
use vertigo::struct_mut::{CounterMut, HashMapMut};

#[derive(Clone)]
pub struct CallbackManager<V> {
    next_id: Rc<CounterMut>,
    data: Rc<HashMapMut<u32, Rc<dyn Fn(&V)>>>,
}

impl<V> CallbackManager<V> {
    pub fn new() -> CallbackManager<V> {
        CallbackManager {
            next_id: Rc::new(CounterMut::new(1)),
            data: Rc::new(HashMapMut::new()),
        }
    }

    pub fn set<F: Fn(&V) + 'static>(&self, callback: F) -> u32 {
        let next_id = self.next_id.get_next();
        self.data.insert(next_id, Rc::new(callback));
        next_id
    }

    pub fn get(&self, callback_id: u32) -> Option<Rc<dyn Fn(&V)>> {
        self.data.get(&callback_id)
    }

    pub fn remove(&self, callback_id: u32) -> Option<Rc<dyn Fn(&V)>> {
        self.data.remove(&callback_id)
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
    next_id: Rc<CounterMut>,
    data: Rc<HashMapMut<u32, Rc<dyn Fn(V)>>>,
}

impl<V> CallbackManagerOwner<V> {
    pub fn new() -> CallbackManagerOwner<V> {
        CallbackManagerOwner {
            next_id: Rc::new(CounterMut::new(1)),
            data: Rc::new(HashMapMut::new()),
        }
    }

    pub fn set(&self, callback: impl Fn(V) + 'static) -> u32 {
        let next_id = self.next_id.get_next();
        self.data.insert(next_id, Rc::new(callback));
        next_id
    }

    fn get(&self, callback_id: u32) -> Option<Rc<dyn Fn(V)>> {
        self.data.get(&callback_id)
    }

    pub fn remove(&self, callback_id: u32) {
        self.data.remove(&callback_id);
    }

    pub fn trigger(&self, callback_id: u32, value: V) {
        let callback = self.get(callback_id);

        match callback {
            Some(callback) => {
                callback(value);
            }
            None => {
                log::error!("Missing callback id {} ", callback_id);
            }
        }
    }
}


#[derive(Clone)]
pub struct CallbackManagerOnce<V> {
    next_id: Rc<CounterMut>,
    data: Rc<HashMapMut<u32, Box<dyn FnOnce(V)>>>,
}

impl<V> CallbackManagerOnce<V> {
    pub fn new() -> CallbackManagerOnce<V> {
        CallbackManagerOnce {
            next_id: Rc::new(CounterMut::new(1)),
            data: Rc::new(HashMapMut::new()),
        }
    }

    pub fn set(&self, callback: impl FnOnce(V) + 'static) -> u32 {
        let next_id = self.next_id.get_next();
        self.data.insert(next_id, Box::new(callback));
        next_id
    }

    pub fn remove(&self, callback_id: u32) -> Option<Box<dyn FnOnce(V)>> {
        self.data.remove(&callback_id)
    }

    #[allow(dead_code)]
    pub fn trigger(&self, callback_id: u32, value: V) {
        let callback = self.data.remove(&callback_id);

        match callback {
            Some(callback) => {
                callback(value);
            }
            None => {
                log::error!("Missing callback id {} ", callback_id);
            }
        }
    }
}
