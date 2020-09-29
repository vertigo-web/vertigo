use virtualdom::{
    computed::{
        Dependencies::Dependencies, Value::Value
    }
};

pub struct SimpleCounter {
    pub counter: Value<u32>,
}

impl SimpleCounter {
    pub fn new(root: &Dependencies) -> SimpleCounter {
        SimpleCounter {
            counter: Value::new(root.clone(), 0)
        }
    }

    pub fn increment(&self) {
        self.counter.setValue(*self.counter.getValue() + 1);
    }
}

