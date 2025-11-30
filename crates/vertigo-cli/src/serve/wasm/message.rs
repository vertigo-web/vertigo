use vertigo::dev::{command::DriverDomCommand, CallbackId, SsrFetchRequest, SsrFetchResponse};

#[derive(Debug)]
pub enum Message {
    TimeoutAndSendResponse,
    DomUpdate(Vec<DriverDomCommand>),
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
