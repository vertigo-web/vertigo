use std::{collections::HashMap, hash::Hash, sync::Arc};
use vertigo::JsJson;

use crate::serve::html::RequestBody;

#[derive(Debug)]
pub enum Message {
    TimeoutAndSendResponse,
    DomUpdate(JsJson),
    Panic(Option<String>),
    SetTimeoutZero {
        callback_id: u64,
    },
    FetchRequest {
        callback_id: u64,
        request: FetchRequest,
    },
    FetchResponse {
        request: Arc<FetchRequest>,
        response: FetchResponse,
    },
    PlainResponse(String),
}

#[derive(Debug, Eq)]
pub struct FetchRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<RequestBody>,
}

impl PartialEq for FetchRequest {
    fn eq(&self, other: &Self) -> bool {
        self.method == other.method
            && self.url == other.url
            && self.headers == other.headers
            && self.body == other.body
    }
}
impl Hash for FetchRequest {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.method.hash(state);
        self.url.hash(state);
        for (key, value) in self.headers.iter() {
            key.hash(state);
            value.hash(state);
        }
        self.body.hash(state);
    }
}

#[derive(Clone, Debug)]
pub struct FetchResponse {
    pub success: bool,
    pub status: u32,
    pub body: RequestBody,
}

pub enum CallWebsocketResult {
    TimeoutSet { time: u32, callback_id: u64 },
    NoResult,
}
