use virtualdom::{
    computed::{
        Computed,
        Dependencies,
        Value
    }
};
// use virtualdom::vdom::StateBox::StateBox;

pub struct State {
    pub counter: Value<u32>,
}

impl State {
    pub fn new(root: &Dependencies) -> Computed<State> {
        root.newComputedFrom(
            State {
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

