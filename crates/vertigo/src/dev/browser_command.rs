use vertigo_macro::AutoJsJson;

use crate::{JsJson, JsJsonContext, JsJsonDeserialize};

pub fn decode_json<T: JsJsonDeserialize>(json: JsJson) -> T {
    T::from_json(JsJsonContext::new(""), json).unwrap()
}

#[derive(AutoJsJson, Debug)]
pub enum BrowserCommand {
    FetchCacheGet,
}

pub mod browser_response {
    use vertigo_macro::AutoJsJson;

    #[derive(AutoJsJson)]
    pub struct FetchCacheGet {
        pub data: Option<String>,
    }
}
