use virtualdom::{
    computed::{
        Dependencies::Dependencies, Value::Value
    }
};
use virtualdom::vdom::StateBox::StateBox;

pub struct SimpleCounter {
    pub counter: Value<u32>,
}

impl SimpleCounter {
    pub fn new(root: &Dependencies) -> StateBox<SimpleCounter> {
        StateBox::new(
            root,
            SimpleCounter {
                counter: Value::new(root.clone(), 0)
            }
        )
    }

    pub fn increment(&self) {
        self.counter.setValue(*self.counter.getValue() + 1);
    }

    pub fn decrement(&self) {
        self.counter.setValue(*self.counter.getValue() - 1);
    }
}

