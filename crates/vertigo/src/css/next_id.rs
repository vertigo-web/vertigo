use crate::struct_mut::ValueMut;

pub struct NextId {
    counter: ValueMut<u64>,
}

impl NextId {
    pub fn new() -> NextId {
        NextId {
            counter: ValueMut::new(0),
        }
    }

    pub fn get_next_id(&self) -> u64 {
        let current = self.counter.get() + 1;
        self.counter.set(current);
        current
    }

    #[cfg(test)]
    pub fn current(&self) -> u64 {
        self.counter.get()
    }
}
