use virtualdom::{
    computed::{
        Computed,
        Dependencies,
        Value
    }
};
// use virtualdom::vdom::StateBox::StateBox;

pub struct SimpleCounter {
    pub counter: Value<u32>,
}

impl SimpleCounter {
    pub fn new(root: &Dependencies) -> Computed<SimpleCounter> {
        root.newComputedFrom(
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

