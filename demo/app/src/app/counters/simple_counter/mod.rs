use vertigo::{Value, DomElement, transaction};

mod render;

#[derive(Clone)]
pub struct State {
    pub counter: Value<u32>,
}

impl State {
    pub fn component(counter: &Value<u32>) -> DomElement {
        let state = State {
            counter: counter.clone(),
        };

        render::render(state)
    }

    pub fn increment(&self) {
        transaction(|context|{
            self.counter.set(self.counter.get(context) + 1);
        });
    }

    pub fn decrement(&self) {
        transaction(|context|{
            self.counter.set(self.counter.get(context) - 1);
        });
    }
}
