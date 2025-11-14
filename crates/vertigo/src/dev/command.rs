use crate::SsrFetchResponse;
use crate::{
    dev::callback_id::CallbackId, JsJson, JsJsonContext, JsJsonDeserialize, SsrFetchRequest,
};
use vertigo_macro::AutoJsJson;

pub fn decode_json<T: JsJsonDeserialize>(json: JsJson) -> T {
    T::from_json(JsJsonContext::new(""), json).unwrap()
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
}

pub mod response_browser {
    use vertigo_macro::AutoJsJson;

    #[derive(AutoJsJson)]
    pub struct FetchCacheGet {
        pub data: Option<String>,
    }
}

#[derive(AutoJsJson, Debug)]
pub enum CommandForWasm {
    FetchExecResponse {
        response: SsrFetchResponse,
        callback: CallbackId,
    },
}
