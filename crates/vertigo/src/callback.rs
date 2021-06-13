use std::cmp::PartialEq;
use std::fmt::{self, Debug};
use std::rc::Rc;

fn get_unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static EQ_COUNTER:AtomicU64 = AtomicU64::new(1);
    EQ_COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone)]
pub struct Callback<P> {
    id: u64,
    pub fun: Rc<dyn Fn(P) -> ()>,
}

impl<P> Callback<P> {
    pub fn new<F: Fn(P) -> () + 'static>(fun: F) -> Callback<P> {
        Callback {
            id: get_unique_id(),
            fun: Rc::new(fun)
        }
    }

    pub fn run(&self, arg: P) {
        let Self { fun, .. } = self;
        fun(arg);
    }
}

impl<P> PartialEq for Callback<P> {
    fn eq(&self, other: &Callback<P>) -> bool {
        self.id == other.id
    }
}

impl<P> Debug for Callback<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Callback")
            .field("id", &self.id)
            .field("value", &"---")
            .finish()
    }
}

