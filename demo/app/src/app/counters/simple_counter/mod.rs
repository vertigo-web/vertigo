use vertigo::{Value, VDomComponent};

mod render;

#[derive(Clone)]
pub struct State {
    pub counter: Value<u32>,
}

impl State {
    pub fn component(counter: &Value<u32>) -> VDomComponent {
        let state = State {
            counter: counter.clone(),
        };

        VDomComponent::from(state, render::render)
    }

    pub fn increment(&self) {
        self.counter.set(self.counter.get() + 1);
    }

    pub fn decrement(&self) {
        self.counter.set(self.counter.get() - 1);
    }
}
