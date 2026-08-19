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

impl<T: Clone> EventEmitter<T> {
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

    /// True when nothing is listening, so the caller can skip preparing a value to emit.
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn trigger(&self, value: &T) {
        // Emitters on hot paths (every DOM command, every `Value` write) usually have no
        // listeners at all, so do not snapshot the callback list for them.
        if self.list.is_empty() {
            return;
        }

        let callback_list = self
            .list
            .map(|state| state.values().cloned().collect::<Vec<_>>());

        for callback in callback_list {
            callback(value.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use super::EventEmitter;

    /// Counts how often the payload is cloned on its way to the listeners.
    #[derive(Debug)]
    struct Counted {
        clones: Rc<Cell<usize>>,
    }

    impl Counted {
        fn new(clones: &Rc<Cell<usize>>) -> Counted {
            Counted {
                clones: clones.clone(),
            }
        }
    }

    impl Clone for Counted {
        fn clone(&self) -> Self {
            self.clones.set(self.clones.get() + 1);

            Counted {
                clones: self.clones.clone(),
            }
        }
    }

    #[test]
    fn trigger_without_listeners_does_not_touch_the_payload() {
        let clones = Rc::new(Cell::new(0));
        let emitter = EventEmitter::<Counted>::default();

        assert!(emitter.is_empty());

        emitter.trigger(&Counted::new(&clones));

        assert_eq!(clones.get(), 0);
    }

    #[test]
    fn trigger_clones_the_payload_once_per_listener() {
        let clones = Rc::new(Cell::new(0));
        let calls = Rc::new(Cell::new(0));
        let emitter = EventEmitter::<Counted>::default();

        let _first = emitter.add({
            let calls = calls.clone();
            move |_| calls.set(calls.get() + 1)
        });
        let _second = emitter.add({
            let calls = calls.clone();
            move |_| calls.set(calls.get() + 1)
        });

        assert!(!emitter.is_empty());

        emitter.trigger(&Counted::new(&clones));

        assert_eq!((calls.get(), clones.get()), (2, 2));
    }
}
