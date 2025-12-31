use vertigo::dev::{CallbackId, SsrFetchRequest, SsrFetchResponse, command::DriverDomCommand};

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
