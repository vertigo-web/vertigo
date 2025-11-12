use std::rc::Rc;

use vertigo_macro::store;

use crate::dev::browser_command::{browser_response, decode_json, BrowserCommand};
use crate::external_api::safe_wrappers;
use crate::{driver_module::api::api_arguments, JsJson, JsJsonSerialize, JsValue, SsrFetchCache};

#[store]
pub fn api_browser_command() -> Rc<ApiBrowserCommand> {
    Rc::new(ApiBrowserCommand {})
}

fn exec_command(command: BrowserCommand) -> JsJson {
    let arg_ptr = JsValue::Json(command.to_json()).to_ptr_long();
    let response = safe_wrappers::safe_dom_access(arg_ptr);
    let response = api_arguments().get_by_long_ptr(response);

    let JsValue::Json(response) = response else {
        unreachable!();
    };

    response
}

pub struct ApiBrowserCommand {}

impl ApiBrowserCommand {
    pub fn fetch_cache_get(&self) -> SsrFetchCache {
        let response = exec_command(BrowserCommand::FetchCacheGet);

        let response = decode_json::<browser_response::FetchCacheGet>(response);

        match response.data {
            Some(data) => {
                let json = JsJson::from_string(&data);

                match json {
                    Ok(json) => decode_json::<SsrFetchCache>(json),
                    Err(message) => {
                        log::error!("ApiBrowserCommand -> fetch_cache_get -> error = {message}");
                        SsrFetchCache::empty()
                    }
                }
            }
            None => SsrFetchCache::empty(),
        }
    }
}
