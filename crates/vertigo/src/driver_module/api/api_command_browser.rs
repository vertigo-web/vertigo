use std::rc::Rc;

use vertigo_macro::store;

use crate::{CallbackId, SsrFetchRequest};
use crate::dev::command::{response_browser, decode_json, CommandBrowser};
use crate::external_api::safe_wrappers;
use crate::{driver_module::api::api_arguments, JsJson, JsJsonSerialize, JsValue, SsrFetchCache};

#[store]
pub fn api_command_browser() -> Rc<ApiCommandBrowser> {
    Rc::new(ApiCommandBrowser {})
}

fn exec_command(command: CommandBrowser) -> JsJson {
    let arg_ptr = JsValue::Json(command.to_json()).to_ptr_long();
    let response = safe_wrappers::safe_dom_access(arg_ptr);
    let response = api_arguments().get_by_long_ptr(response);

    let JsValue::Json(response) = response else {
        unreachable!();
    };

    response
}

pub struct ApiCommandBrowser {}

impl ApiCommandBrowser {
    pub fn fetch_cache_get(&self) -> SsrFetchCache {
        let response = exec_command(CommandBrowser::FetchCacheGet);

        let response = decode_json::<response_browser::FetchCacheGet>(response);

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

    pub fn fetch_exec(&self, request: SsrFetchRequest, callback: CallbackId) {
        exec_command(CommandBrowser::FetchExec { request, callback });
    }

    //...
}
