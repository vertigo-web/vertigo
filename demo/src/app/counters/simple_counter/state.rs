use std::cmp::PartialEq;
use vertigo::{Computed, Driver, Value};

#[derive(PartialEq)]
pub struct State {
    pub counter: Value<u32>,
}

impl State {
    pub fn new(driver: &Driver) -> Computed<State> {
        driver.new_computed_from(
            State {
                counter: driver.new_value(0),
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

