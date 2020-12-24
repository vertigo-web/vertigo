
pub struct NextId {
    counter: u64,
}

impl NextId {
    pub fn new() -> NextId {
        NextId {
            counter: 0,
        }
    }

    pub fn get_next_id(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }

    #[cfg(test)]
    pub fn current(&self) -> u64 {
        self.counter
    }
}
