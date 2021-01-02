use core::{
    fmt::Debug,
    cmp::{PartialOrd, Ord},
};

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, PartialOrd, Ord)]
pub struct GraphId {
    pub id: u64,
}

impl Default for GraphId {
    fn default() -> Self {
        Self {
            id: GraphId::get_unique_id()
        }
    }
}

impl GraphId {
    fn get_unique_id() -> u64 {
        use core::sync::atomic::{AtomicU64, Ordering};
        static COUNTER:AtomicU64 = AtomicU64::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }
}