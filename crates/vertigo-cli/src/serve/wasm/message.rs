use vertigo::{JsJson, SsrFetchRequest, SsrFetchResponse};

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
        request: SsrFetchRequest,
    },
    FetchResponse {
        request: SsrFetchRequest,
        response: SsrFetchResponse,
    },
    SetStatus(u16),
}

#[derive(Debug, PartialEq)]
pub enum CallWebsocketResult {
    TimeoutSet { time: u32, callback_id: u64 },
    NoResult,
}
