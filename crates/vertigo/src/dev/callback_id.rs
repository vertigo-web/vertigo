use std::sync::atomic::{AtomicU64, Ordering};

use crate::{JsJson, JsJsonContext, JsJsonDeserialize, JsJsonNumber, JsJsonSerialize};

#[derive(PartialEq, Eq, Hash, Clone, Debug, Copy)]
pub struct CallbackId(u64);

#[cfg(test)]
use {dashmap::DashMap, std::sync::LazyLock, std::thread::ThreadId};

#[cfg(not(test))]
static COUNTER: AtomicU64 = AtomicU64::new(1);

#[cfg(test)]
// For tests, keep separate counter in every thread
static COUNTER: LazyLock<DashMap<ThreadId, AtomicU64>> = LazyLock::new(DashMap::new);

#[cfg(not(test))]
impl CallbackId {
    #[allow(clippy::new_without_default)]
    pub fn new() -> CallbackId {
        CallbackId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn from_u64(id: u64) -> Self {
        Self(id)
    }
}

#[cfg(test)]
impl CallbackId {
    #[allow(clippy::new_without_default)]
    pub fn new() -> CallbackId {
        let tid = std::thread::current().id();
        let counter = COUNTER.entry(tid).or_insert_with(|| AtomicU64::new(1));
        CallbackId(counter.fetch_add(1, Ordering::Relaxed))
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn from_u64(id: u64) -> Self {
        Self(id)
    }
}

impl JsJsonSerialize for CallbackId {
    fn to_json(self) -> JsJson {
        JsJson::Number(JsJsonNumber(self.0 as f64))
    }
}

impl JsJsonDeserialize for CallbackId {
    fn from_json(_context: JsJsonContext, json: JsJson) -> Result<Self, JsJsonContext> {
        if let JsJson::Number(JsJsonNumber(value)) = json {
            return Ok(CallbackId::from_u64(value as u64));
        }

        Err(JsJsonContext::new(format!(
            "Expected Number, received={}",
            json.typename()
        )))
    }
}
