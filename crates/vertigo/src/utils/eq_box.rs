use std::cmp::PartialEq;
use std::ops::Deref;
use std::fmt::{self, Debug};

fn get_unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
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

    pub fn into_inner(self) -> T {
        self.value
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

impl<T> Debug for EqBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EqBox")
            .field("id", &self.id)
            .field("value", &"---")
            .finish()
    }
}
