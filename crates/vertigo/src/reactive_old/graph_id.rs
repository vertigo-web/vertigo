use std::cmp::{Ord, PartialOrd};

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, PartialOrd, Ord)]
pub enum GraphIdKind {
    Value,
    Computed,
    Client,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, PartialOrd, Ord)]
pub struct GraphId(GraphIdKind, u64);

impl GraphId {
    pub fn new_value() -> GraphId {
        GraphId(GraphIdKind::Value, GraphId::get_unique_id())
    }

    pub fn new_computed() -> GraphId {
        GraphId(GraphIdKind::Computed, GraphId::get_unique_id())
    }

    pub fn new_client() -> GraphId {
        GraphId(GraphIdKind::Client, GraphId::get_unique_id())
    }

    pub fn id(&self) -> u64 {
        self.1
    }

    pub fn get_type(&self) -> GraphIdKind {
        self.0
    }

    #[cfg(test)]
    pub fn new_for_test(kind: GraphIdKind, id: u64) -> GraphId {
        GraphId(kind, id)
    }
}

impl GraphId {
    fn get_unique_id() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }
}
