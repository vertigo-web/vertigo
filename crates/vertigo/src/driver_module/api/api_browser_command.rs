use std::rc::Rc;

use vertigo_macro::{AutoJsJson, store};

use crate::{JsJsonContext, JsJsonDeserialize};
use crate::{JsJson, JsJsonSerialize, JsValue, SsrFetchCache, dev::BrowserCommand, driver_module::api::api_arguments};
use crate::external_api::safe_wrappers;


#[store]
pub fn api_browser_command() -> Rc<ApiBrowserCommand> {

    Rc::new(ApiBrowserCommand {})
}

fn exec_command(command: BrowserCommand) -> JsJson {
    let arg_ptr = JsValue::Json(command.to_json()).to_ptr_long();
    let reponse = safe_wrappers::safe_dom_access(arg_ptr);
    let response = api_arguments().get_by_long_ptr(reponse);

    let JsValue::Json(response) = response else {
        unreachable!();
    };

    response
}

pub struct ApiBrowserCommand {

}

impl ApiBrowserCommand {

    pub fn fetch_cache_get(&self) -> SsrFetchCache {
        let response = exec_command(BrowserCommand::FetchCacheGet);

        #[derive(AutoJsJson)]
        struct Response {
            data: Option<String>,
        }

        let response = Response::from_json(JsJsonContext::new(""), response).unwrap();  //TODO - Lepiej obsłuzyć

        match response.data {
            Some(data) => {
                let json = JsJson::from_string(&data).unwrap();
                let cache = SsrFetchCache::from_json(JsJsonContext::new(""), json);
                
                match cache {
                    Ok(result) => return result,
                    Err(err) => {
                        panic!("fetch_cache_get -> {:?}", err)
                    }
                }
            },
            None => {
                SsrFetchCache::empty()
            }
        }
    }
}