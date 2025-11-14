use vertigo::CallbackId;
use vertigo::{JsJson, SsrFetchRequest, SsrFetchResponse};

#[derive(Debug)]
pub enum Message {
    TimeoutAndSendResponse,
    DomUpdate(JsJson),
    Panic(Option<String>),
    SetTimeoutZero {
        callback: CallbackId,
    },
    FetchRequest {
        callback: CallbackId,
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
    TimeoutSet { time: u32, callback: CallbackId },
    NoResult,
}
