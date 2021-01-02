use core::cmp::PartialEq;
use core::ops::Deref;

fn get_unique_id() -> u64 {
    use core::sync::atomic::{AtomicU64, Ordering};
    static EQ_COUNTER:AtomicU64 = AtomicU64::new(1);
    EQ_COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone)]
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

impl<T> Deref for EqBox<T> {
    type Target = T;
    fn deref(&self) -> &T { &self.value }
}

impl<T> PartialEq for EqBox<T> {
    fn eq(&self, other: &EqBox<T>) -> bool {
        self.id == other.id
    }

    fn ne(&self, other: &EqBox<T>) -> bool {
        self.id != other.id
    }
}
