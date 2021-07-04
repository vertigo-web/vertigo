use std::rc::Rc;
use vertigo::utils::BoxRefCell;

#[derive(Clone)]
pub struct CounterRc {
    counter: Rc<BoxRefCell<u64>>,
}

impl CounterRc {
    pub fn new(init: u64) -> CounterRc {
        CounterRc {
            counter: Rc::new(BoxRefCell::new(init, "CounterRc")),
        }
    }

    pub fn get_next(&self) -> u64 {
        self.counter.change((), |state, _| {
            let id = *state;
            *state += 1;
            id
        })
    }
}