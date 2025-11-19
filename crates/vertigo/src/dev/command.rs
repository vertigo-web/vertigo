use crate::SsrFetchResponse;
use crate::{
    dev::callback_id::CallbackId, JsJson, JsJsonContext, JsJsonDeserialize, SsrFetchRequest,
};
use vertigo_macro::AutoJsJson;

pub fn decode_json<T: JsJsonDeserialize>(json: JsJson) -> Result<T, JsJsonContext> {
    T::from_json(JsJsonContext::new(""), json)
}

#[derive(AutoJsJson, Debug)]
pub enum CommandForBrowser {
    FetchCacheGet,
    FetchExec {
        request: SsrFetchRequest,
        callback: CallbackId,
    },
    SetStatus {
        status: u16,
    },
    IsBrowser,
    GetDateNow,

    WebsocketRegister {
        host: String,
        callback: CallbackId,
    },
    WebsocketUnregister {
        callback: CallbackId,
    },
    WebsocketSendMessage {
        callback: CallbackId,
        message: String,
    },

    TimerSet {
        callback: CallbackId,
        duration: u32,
        kind: TimerKind,
    },
    TimerClear {
        callback: CallbackId,
    },
}

#[derive(AutoJsJson, Debug)]
pub enum TimerKind {
    Timeout,
    Interval,
}

pub mod browser_response {
    use vertigo_macro::AutoJsJson;

    use crate::dev::InstantType;

    #[derive(AutoJsJson)]
    pub struct FetchCacheGet {
        pub data: Option<String>,
    }

    #[derive(AutoJsJson)]
    pub struct IsBrowser {
        pub value: bool,
    }

    #[derive(AutoJsJson)]
    pub struct GetDateNow {
        pub value: InstantType,
    }
}

#[derive(AutoJsJson, Debug)]
pub enum WebsocketMessageFromBrowser {
    Connected,
    Message { message: String },
    Disconnected,
}

#[derive(AutoJsJson, Debug)]
pub enum CommandForWasm {
    FetchExecResponse {
        response: SsrFetchResponse,
        callback: CallbackId,
    },

    Websocket {
        callback: CallbackId,
        message: WebsocketMessageFromBrowser,
    },

    TimerCall {
        callback: CallbackId,
    },
}
