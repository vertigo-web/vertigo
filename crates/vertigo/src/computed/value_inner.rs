use super::{GraphId, struct_mut::ValueMut};
use crate::{DropResource, driver_module::event_emitter::EventEmitter};

pub struct ValueInner<T: PartialEq + Clone + 'static> {
    pub id: GraphId,
    value: ValueMut<T>,
    events: EventEmitter<T>,
}

impl<T: PartialEq + Clone + 'static> ValueInner<T> {
    pub fn new(value: T) -> ValueInner<T> {
        ValueInner {
            id: GraphId::new_value(),
            value: ValueMut::new(value),
            events: EventEmitter::default(),
        }
    }

    #[must_use]
    pub fn set(&self, value: T) -> bool {
        // The clone only exists to hand the new value to the listeners, so skip it when
        // there are none - otherwise every write deep-copies whatever is stored.
        if self.events.is_empty() {
            return self.value.set_if_changed(value);
        }

        let change = self.value.set_if_changed(value.clone());

        if change {
            self.events.trigger(&value);
        }

        change
    }

    pub fn add_event(&self, callback: impl Fn(T) + 'static) -> DropResource {
        self.events.add(callback)
    }

    pub fn get(&self) -> T {
        self.value.get()
    }
}
