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
    /// The render read a cookie the app may read on the server
    CookieRead,
    /// The render read the languages of the browser
    LanguagesRead,
}
