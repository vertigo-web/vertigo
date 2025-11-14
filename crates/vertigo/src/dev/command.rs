use vertigo_macro::AutoJsJson;
use crate::{JsJson, JsJsonContext, JsJsonDeserialize, SsrFetchRequest, dev::callback_id::CallbackId};
use crate::{SsrFetchResponse};

pub fn decode_json<T: JsJsonDeserialize>(json: JsJson) -> T {
    T::from_json(JsJsonContext::new(""), json).unwrap()
}


#[derive(AutoJsJson, Debug)]
pub enum CommandBrowser {
    FetchCacheGet,
    FetchExec {
        request: SsrFetchRequest,
        callback: CallbackId,
    }
}

pub mod response_browser {
    use vertigo_macro::AutoJsJson;

    #[derive(AutoJsJson)]
    pub struct FetchCacheGet {
        pub data: Option<String>,
    }
}

/*
Obecną metodę wasm_callback po prostu powolutku wygaszać

CallbackId
serializować jako dwie liczby, u32, u32
wtedy nie powinno być problemu

js musi tylko odebrać taką złozoną liczbę i ją odesłać w odpowiednim momencie
*/

#[derive(AutoJsJson, Debug)]
pub enum CommandWasm {
    FetchExecResponse {
        response: SsrFetchResponse,
        callback: CallbackId,
    }
}

