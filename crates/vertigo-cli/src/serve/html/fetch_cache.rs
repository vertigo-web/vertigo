use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use parking_lot::RwLock;
use vertigo::{SsrFetchRequest, SsrFetchResponse};

pub struct FetchCache {
    pub fetch_waiting: HashMap<SsrFetchRequest, Vec<u64>>,
    pub fetch_cache: BTreeMap<SsrFetchRequest, SsrFetchResponse>,
}

impl FetchCache {
    pub fn new() -> Arc<RwLock<FetchCache>> {
        Arc::new(RwLock::new(FetchCache {
            fetch_waiting: HashMap::new(),
            fetch_cache: BTreeMap::new(),
        }))
    }
}
