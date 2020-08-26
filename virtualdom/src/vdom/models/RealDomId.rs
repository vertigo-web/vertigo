
const ROOT_ID: u64 = 1;
const START_ID: u64 = 2;

fn get_unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER:AtomicU64 = AtomicU64::new(START_ID);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone, Debug)]
pub struct RealDomId {
    id: u64,
}

impl RealDomId {
    pub fn root() -> RealDomId {
        RealDomId {
            id: ROOT_ID
        }
    }

    pub fn new() -> RealDomId {
        RealDomId {
            id: get_unique_id()
        }
    }
}

impl std::fmt::Display for RealDomId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "RealDomNodeId={}", self.id)
    }
}