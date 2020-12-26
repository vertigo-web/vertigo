use std::cmp::PartialEq;

fn get_unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static EQ_COUNTER:AtomicU64 = AtomicU64::new(1);
    EQ_COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub struct EqBox<T> {
    id: u64,
    pub value: T,
}

impl<T> EqBox<T> {
    pub fn new(value: T) -> EqBox<T> {
        EqBox {
            id: get_unique_id(),
            value
        }
    }
}

impl<T> PartialEq for EqBox<T> {
    fn eq(&self, other: &EqBox<T>) -> bool {
        self.id == other.id
    }

    fn ne(&self, other: &EqBox<T>) -> bool {
        self.id != other.id
    }
}