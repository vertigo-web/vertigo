use std::rc::Rc;

use vertigo_macro::store;

use crate::dev::command::{browser_response, decode_json, CommandForBrowser};
use crate::dev::InstantType;
use crate::external_api::safe_wrappers;
use crate::{driver_module::api::api_arguments, JsJson, JsJsonSerialize, JsValue, SsrFetchCache};
use crate::{CallbackId, SsrFetchRequest};

#[store]
pub fn api_browser_command() -> Rc<CommandForBrowserApi> {
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
        let command_result = exec_command(CommandForBrowser::FetchCacheGet);

        // Deserialize from WASM-JS bridge
        decode_json::<browser_response::FetchCacheGet>(command_result)
            .inspect_err(|err| {
                log::error!("fetch_cache_get -> decode error = {err}");
            })
            .ok()
            // Response maybe empty
            .and_then(|response| response.data)
            // Decode response from string
            .and_then(|data| {
                JsJson::from_string(&data)
                    .inspect_err(|message| {
                        log::error!("fetch_cache_get -> error = {message}");
                    })
                    .ok()
            })
            // Deserialize the response itself
            .and_then(|json| {
                decode_json::<SsrFetchCache>(json)
                    .inspect_err(|err| {
                        log::error!("fetch_cache_get -> decode error = {err}");
                    })
                    .ok()
            })
            .unwrap_or_else(SsrFetchCache::empty)
    }

    pub fn fetch_exec(&self, request: SsrFetchRequest, callback: CallbackId) {
        exec_command(CommandForBrowser::FetchExec { request, callback });
    }

    pub fn set_status(&self, status: u16) {
        exec_command(CommandForBrowser::SetStatus { status });
    }

    pub fn is_browser(&self) -> bool {
        let response = exec_command(CommandForBrowser::IsBrowser);
        let response = decode_json::<browser_response::IsBrowser>(response);
        match response {
            Ok(response) => response.value,
            Err(err) => {
                log::error!("is_browser -> decode error = {err}");
                false
            }
        }
    }

    pub fn get_date_now(&self) -> InstantType {
        let response = exec_command(CommandForBrowser::GetDateNow);
        let response = decode_json::<browser_response::GetDateNow>(response);
        match response {
            Ok(response) => response.value,
            Err(err) => {
                log::error!("get_date_now -> decode error = {err}");
                InstantType::default()
            }
        }
    }

    pub fn websocket_register_callback(&self, host: &str, callback: CallbackId) {
        exec_command(CommandForBrowser::WebsocketRegister {
            host: host.to_string(),
            callback,
        });
    }

    pub fn websocket_unregister_callback(&self, callback: CallbackId) {
        exec_command(CommandForBrowser::WebsocketUnregister { callback });
    }

    pub fn websocket_send_message(&self, callback: CallbackId, message: &str) {
        exec_command(CommandForBrowser::WebsocketSendMessage {
            callback,
            message: message.to_string(),
        });
    }

    //....
}
