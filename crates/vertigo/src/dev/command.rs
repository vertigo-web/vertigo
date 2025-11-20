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

    LocationCallback {
        target: LocationTarget,
        mode: LocationCallbackMode,
        callback: CallbackId,
    },
    LocationSet {
        target: LocationTarget,
        mode: LocationSetMode,
        value: String,
    },
    LocationGet {
        target: LocationTarget,
    },

    CookieGet {
        name: String,
    },
    CookieSet {
        name: String,
        value: String,
        expires_in: u64,
    },
    CookieJsonGet {
        name: String,
    },
    CookieJsonSet {
        name: String,
        value: JsJson,
        expires_in: u64,
    },

    GetEnv {
        name: String,
    },
}

#[derive(AutoJsJson, Debug, Clone, Copy, PartialEq)]
pub enum LocationTarget {
    Hash,
    History,
}

#[derive(AutoJsJson, Debug, Clone, Copy)]
pub enum LocationCallbackMode {
    Add,
    Remove,
}

#[derive(AutoJsJson, Debug, Clone, Copy)]
pub enum LocationSetMode {
    Push,
    Replace,
}

#[derive(AutoJsJson, Debug)]
pub enum TimerKind {
    Timeout,
    Interval,
}

pub mod browser_response {
    use vertigo_macro::AutoJsJson;

    use crate::{dev::InstantType, JsJson};

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

    #[derive(AutoJsJson)]
    pub struct LocationGet {
        pub value: String,
    }

    #[derive(AutoJsJson)]
    pub struct CookieGet {
        pub value: String,
    }

    #[derive(AutoJsJson)]
    pub struct CookieJsonGet {
        pub value: JsJson,
    }

    #[derive(AutoJsJson)]
    pub struct GetEnv {
        pub value: Option<String>,
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

    LocationCall {
        callback: CallbackId,
        value: String,
    },
}
