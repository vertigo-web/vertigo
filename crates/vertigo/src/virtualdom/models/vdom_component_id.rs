use std::cmp::{Ord, PartialOrd};

fn get_unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, PartialOrd, Ord)]
pub struct VDomComponentId(u64);

impl Default for VDomComponentId {
    fn default() -> Self {
        VDomComponentId(get_unique_id())
    }
}

