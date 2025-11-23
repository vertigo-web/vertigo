use crate::driver_module::StaticString;
use crate::{
    dev::callback_id::CallbackId, JsJson, JsJsonContext, JsJsonDeserialize, SsrFetchRequest,
};
use crate::{DomId, SsrFetchResponse};
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

    ConsoleLog {
        kind: ConsoleLogLevel,
        message: String,
        arg2: String,
        arg3: String,
        arg4: String,
    },

    TimezoneOffset,
    HistoryBack,
    GetRandom {
        min: u32,
        max: u32,
    },

    /// Execute JavaScript API calls (for js! macro)
    JsApiCall {
        commands: Vec<JsApiCommand>,
    },

    DomBulkUpdate {
        list: Vec<DriverDomCommand>,
    },
}

#[derive(AutoJsJson, Debug, Clone)]
pub enum JsApiCommand {
    Root { name: String },
    RootElement { dom_id: u64 },
    Get { property: String },
    Set { property: String, value: JsJson },
    Call { method: String, args: Vec<JsJson> },
}

#[derive(AutoJsJson, Debug, PartialEq)]
pub enum ConsoleLogLevel {
    Debug,
    Info,
    Log,
    Warn,
    Error,
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

    #[derive(AutoJsJson)]
    pub struct TimezoneOffset {
        pub value: i32,
    }

    #[derive(AutoJsJson)]
    pub struct GetRandom {
        pub value: u32,
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

#[derive(AutoJsJson, Clone, Debug)]
pub enum DriverDomCommand {
    CreateNode {
        id: DomId,
        name: StaticString,
    },
    CreateText {
        id: DomId,
        value: String,
    },
    UpdateText {
        id: DomId,
        value: String,
    },
    SetAttr {
        id: DomId,
        name: StaticString,
        value: String,
    },
    RemoveAttr {
        id: DomId,
        name: StaticString,
    },
    RemoveNode {
        id: DomId,
    },
    RemoveText {
        id: DomId,
    },
    InsertBefore {
        parent: DomId,
        child: DomId,
        ref_id: Option<DomId>,
    },
    InsertCss {
        selector: Option<String>,
        value: String,
    },

    CreateComment {
        id: DomId,
        value: String,
    },
    RemoveComment {
        id: DomId,
    },
    CallbackAdd {
        id: DomId,
        event_name: String,
        callback_id: CallbackId,
    },
    CallbackRemove {
        id: DomId,
        event_name: String,
        callback_id: CallbackId,
    },
}

impl DriverDomCommand {
    pub fn is_event(&self) -> bool {
        matches!(
            self,
            Self::RemoveNode { .. } | Self::RemoveText { .. } | Self::RemoveComment { .. }
        )
    }
}
