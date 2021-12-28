use std::cell::RefCell;

pub struct CounterMut {
    counter: RefCell<u64>,
}

impl CounterMut {
    pub fn new(init: u64) -> CounterMut {
        CounterMut {
            counter: RefCell::new(init),
        }
    }

    pub fn get_next(&self) -> u64 {
        let mut state = self.counter.borrow_mut();
        let id = *state;
        *state += 1;
        id
    }
}
