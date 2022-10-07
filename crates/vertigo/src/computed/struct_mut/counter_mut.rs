use super::inner_value::InnerValue;

pub struct CounterMut {
    counter: InnerValue<u32>,
}

impl CounterMut {
    pub fn new(init: u32) -> CounterMut {
        CounterMut {
            counter: InnerValue::new(init),
        }
    }

    pub fn get_next(&self) -> u32 {
        let state = self.counter.get_mut();
        let id = *state;
        *state += 1;
        id
    }
}
