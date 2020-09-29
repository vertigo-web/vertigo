use virtualdom::{
    computed::{
        Dependencies::Dependencies, Value::Value
    }
};

use crate::simple_counter::simple_counter::SimpleCounter;
use virtualdom::vdom::StateBox::StateBox;

pub struct AppState {
    pub value: Value<u32>,
    pub at: Value<u32>,
    pub counter1: StateBox<SimpleCounter>,
    pub counter2: StateBox<SimpleCounter>,
    pub counter3: StateBox<SimpleCounter>,
}

impl AppState {
    pub fn new(root: &Dependencies) -> AppState {
        AppState {
            value: root.newValue(33),
            at: root.newValue(999),
            counter1: StateBox::new(&root, SimpleCounter::new(&root)),
            counter2: StateBox::new(&root, SimpleCounter::new(&root)),
            counter3: StateBox::new(&root, SimpleCounter::new(&root)),
        }
    }

    pub fn increment(&self) {
        let rr = self.value.getValue();
        self.value.setValue(*rr + 1);
    }

    pub fn decrement(&self) {
        let rr = self.value.getValue();
        self.value.setValue(*rr - 1);
    }
}
