use std::hash::Hash;
const ROOT_ID: u64 = 1;
const START_ID: u64 = 2;

fn get_unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(START_ID);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RealDomId {
    id: u64,
}

impl Default for RealDomId {
    fn default() -> Self {
        Self { id: get_unique_id() }
    }
}

impl RealDomId {
    pub fn root() -> RealDomId {
        RealDomId { id: ROOT_ID }
    }

    pub fn from_u64(id: u64) -> RealDomId {
        RealDomId { id }
    }

    pub fn to_u64(&self) -> u64 {
        self.id
    }
}

impl std::fmt::Display for RealDomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RealDomNodeId={}", self.id)
    }
}
