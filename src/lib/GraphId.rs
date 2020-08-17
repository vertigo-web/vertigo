
#[derive(Hash, PartialEq, Eq, Clone)]
pub struct GraphId {
    pub id: u64,
}

impl GraphId {
    pub fn new() -> GraphId {
        GraphId {
            id: GraphId::get_unique_id()
        }
    }

    fn get_unique_id() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER:AtomicU64 = AtomicU64::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }
}