use std::cell::RefCell;

pub struct CounterMut {
    counter: RefCell<u32>,
}

impl CounterMut {
    pub fn new(init: u32) -> CounterMut {
        CounterMut {
            counter: RefCell::new(init),
        }
    }

    pub fn get_next(&self) -> u32 {
        let mut state = self.counter.borrow_mut();
        let id = *state;
        *state += 1;
        id
    }
}
