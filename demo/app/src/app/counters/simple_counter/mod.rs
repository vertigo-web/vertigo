use std::cmp::PartialEq;
use vertigo::{Driver, Value, VDomComponent};

mod render;

#[derive(PartialEq)]
pub struct State {
    pub counter: Value<u32>,
}

impl State {
    pub fn component(driver: &Driver, counter: &Value<u32>) -> VDomComponent {
        let state = State {
            counter: counter.clone(),
        };

        driver.bind_render(state, render::render)
    }

    pub fn increment(&self) {
        self.counter.set_value(*self.counter.get_value() + 1);
    }

    pub fn decrement(&self) {
        self.counter.set_value(*self.counter.get_value() - 1);
    }
}
