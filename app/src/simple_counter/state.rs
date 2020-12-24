use vertigo::{
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
        root.new_computed_from(
            State {
                counter: Value::new(root.clone(), 0)
            }
        )
    }

    pub fn increment(&self) {
        self.counter.set_value(*self.counter.get_value() + 1);
    }

    pub fn decrement(&self) {
        self.counter.set_value(*self.counter.get_value() - 1);
    }
}

