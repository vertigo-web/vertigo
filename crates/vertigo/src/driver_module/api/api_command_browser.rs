use std::rc::Rc;

use vertigo_macro::store;

use crate::dev::InstantType;
use crate::dev::command::{decode_json, response_browser, CommandForBrowser};
use crate::external_api::safe_wrappers;
use crate::{driver_module::api::api_arguments, JsJson, JsJsonSerialize, JsValue, SsrFetchCache};
use crate::{CallbackId, SsrFetchRequest};

#[store]
pub fn api_command_browser() -> Rc<CommandForBrowserApi> {
    Rc::new(CommandForBrowserApi {})
}

fn exec_command(command: CommandForBrowser) -> JsJson {
    let arg_ptr = JsValue::Json(command.to_json()).to_ptr_long();
    let response = safe_wrappers::safe_dom_access(arg_ptr);
    let response = api_arguments().get_by_long_ptr(response);

    if let JsValue::Json(response) = response {
        return response;
    };

    log::error!("expected json, received {:?}", response.typename());
    JsJson::Null
}

pub struct CommandForBrowserApi {}

impl CommandForBrowserApi {
    pub fn fetch_cache_get(&self) -> SsrFetchCache {
        let response = exec_command(CommandForBrowser::FetchCacheGet);

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
        exec_command(CommandForBrowser::FetchExec { request, callback });
    }

    pub fn set_status(&self, status: u16) {
        exec_command(CommandForBrowser::SetStatus { status });
    }

    pub fn is_browser(&self) -> bool {
        let response = exec_command(CommandForBrowser::IsBrowser);
        let response = decode_json::<response_browser::IsBrowser>(response);
        response.value
    }

    pub fn get_date_now(&self) -> InstantType {
        let response = exec_command(CommandForBrowser::GetDateNow);
        let response = decode_json::<response_browser::GetDateNow>(response);
        response.value
    }
}
