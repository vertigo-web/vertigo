use std::collections::BTreeMap;

use vertigo_macro::AutoJsJson;

use crate::{FetchMethod, JsJson};

#[derive(AutoJsJson, Debug, Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SsrFetchRequestBody {
    None,
    Data { data: JsJson },
}

#[derive(AutoJsJson, Debug, Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SsrFetchRequest {
    pub method: FetchMethod,
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub body: SsrFetchRequestBody,
}

#[derive(AutoJsJson, Debug, Hash, Clone)]
pub enum SsrFetchResponse {
    Ok { status: u32, response: JsJson },
    Err { message: String },
}

#[derive(AutoJsJson, Debug, Hash, Clone, Default)]
pub struct SsrFetchCache {
    pub data: BTreeMap<SsrFetchRequest, SsrFetchResponse>,
}

impl SsrFetchCache {
    pub fn new(data: &BTreeMap<SsrFetchRequest, SsrFetchResponse>) -> SsrFetchCache {
        SsrFetchCache { data: data.clone() }
    }

    pub fn empty() -> SsrFetchCache {
        SsrFetchCache {
            data: BTreeMap::new(),
        }
    }

    pub fn get(&self, request: &SsrFetchRequest) -> Option<&SsrFetchResponse> {
        self.data.get(request)
    }
}
