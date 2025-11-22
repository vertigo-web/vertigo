use std::{
    hash::Hash,
    sync::atomic::{AtomicBool, Ordering},
};

use vertigo_macro::AutoJsJson;

const HTML_ID: u64 = 1;
const HEAD_ID: u64 = 2;
const BODY_ID: u64 = 3;
const START_ID: u64 = 4;

fn get_unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(START_ID);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

// Flags to ensure we have only 1 instance of these elements with predefined id
static HAD_HTML: AtomicBool = AtomicBool::new(false);
static HAD_HEAD: AtomicBool = AtomicBool::new(false);
static HAD_BODY: AtomicBool = AtomicBool::new(false);

#[derive(AutoJsJson, Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DomId(u64);

impl Default for DomId {
    fn default() -> Self {
        Self(get_unique_id())
    }
}

impl DomId {
    pub fn from_name(name: &str) -> DomId {
        let new_id = match name {
            "html" => maybe_static_id("html", &HAD_HTML, HTML_ID),
            "head" => maybe_static_id("head", &HAD_HEAD, HEAD_ID),
            "body" => maybe_static_id("body", &HAD_BODY, BODY_ID),
            _ => get_unique_id(),
        };

        DomId(new_id)
    }

    pub fn root_id() -> DomId {
        DomId(HTML_ID)
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

fn maybe_static_id(name: &'static str, flag: &'static AtomicBool, static_id: u64) -> u64 {
    if flag.load(Ordering::Relaxed) {
        log::error!("Multiple <{name}> elements!");
        get_unique_id()
    } else {
        flag.store(true, Ordering::Relaxed);
        static_id
    }
}
