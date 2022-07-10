use std::hash::Hash;
const ROOT_ID: u64 = 1;
const START_ID: u64 = 2;

fn get_unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(START_ID);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct DomId(u64);

impl Default for DomId {
    fn default() -> Self {
        Self(get_unique_id())
    }
}

impl DomId {
    pub fn root() -> DomId {
        DomId(ROOT_ID)
    }

    pub fn from_u64(id: u64) -> DomId {
        DomId(id)
    }

    pub fn to_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for DomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RealDomNodeId={}", self.0)
    }
}
