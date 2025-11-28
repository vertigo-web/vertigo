use std::rc::Rc;

use vertigo_macro::store;

use crate::command::{
    ConsoleLogLevel, DriverDomCommand, LocationCallbackMode, LocationSetMode, LocationTarget,
    TimerKind,
};
use crate::dev::command::{browser_response, decode_json, CommandForBrowser};
use crate::dev::InstantType;
use crate::external_api::safe_wrappers;
use crate::{driver_module::api::api_arguments, JsJson, JsJsonSerialize, SsrFetchCache};
use crate::{CallbackId, SsrFetchRequest};

#[store]
pub fn api_browser_command() -> Rc<CommandForBrowserApi> {
    Rc::new(CommandForBrowserApi {})
}

pub(crate) fn exec_command(command: CommandForBrowser) -> JsJson {
    let arg_ptr = command.to_json().to_ptr_long();
    let response = safe_wrappers::safe_dom_access(arg_ptr);
    api_arguments().get_by_long_ptr(response)
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

    pub fn timer_set(&self, callback: CallbackId, duration: u32, kind: TimerKind) {
        exec_command(CommandForBrowser::TimerSet {
            callback,
            duration,
            kind,
        });
    }

    pub fn timer_clear(&self, callback: CallbackId) {
        exec_command(CommandForBrowser::TimerClear { callback });
    }

    pub fn location_callback(
        &self,
        target: LocationTarget,
        mode: LocationCallbackMode,
        callback: CallbackId,
    ) {
        exec_command(CommandForBrowser::LocationCallback {
            target,
            mode,
            callback,
        });
    }

    pub fn location_set(&self, target: LocationTarget, mode: LocationSetMode, value: String) {
        exec_command(CommandForBrowser::LocationSet {
            target,
            mode,
            value,
        });
    }

    pub fn location_get(&self, target: LocationTarget) -> String {
        let response = exec_command(CommandForBrowser::LocationGet { target });

        let response = decode_json::<browser_response::LocationGet>(response);
        match response {
            Ok(response) => response.value,
            Err(err) => {
                log::error!("location_get -> decode error = {err}");
                "".into()
            }
        }
    }

    pub fn cookie_set(&self, name: String, value: String, expires_in: u64) {
        exec_command(CommandForBrowser::CookieSet {
            name,
            value,
            expires_in,
        });
    }

    pub fn cookie_json_set(&self, name: String, value: JsJson, expires_in: u64) {
        exec_command(CommandForBrowser::CookieJsonSet {
            name,
            value,
            expires_in,
        });
    }

    pub fn dom_bulk_update(&self, list: Vec<DriverDomCommand>) {
        exec_command(CommandForBrowser::DomBulkUpdate { list });
    }

    pub fn cookie_get(&self, name: String) -> String {
        let response = exec_command(CommandForBrowser::CookieGet { name });

        let response = decode_json::<browser_response::CookieGet>(response);
        match response {
            Ok(response) => response.value,
            Err(err) => {
                log::error!("cookie_get -> decode error = {err}");
                "".into()
            }
        }
    }

    pub fn cookie_json_get(&self, name: String) -> JsJson {
        let response = exec_command(CommandForBrowser::CookieJsonGet { name });

        let response = decode_json::<browser_response::CookieJsonGet>(response);
        match response {
            Ok(response) => response.value,
            Err(err) => {
                log::error!("cookie_get -> decode error = {err}");
                JsJson::Null
            }
        }
    }

    pub fn get_env(&self, name: impl Into<String>) -> Option<String> {
        let response = exec_command(CommandForBrowser::GetEnv { name: name.into() });

        let response = decode_json::<browser_response::GetEnv>(response);
        match response {
            Ok(response) => response.value,
            Err(err) => {
                log::error!("get_env -> decode error = {err}");
                None
            }
        }
    }

    pub fn console_log(
        &self,
        kind: ConsoleLogLevel,
        message: impl Into<String>,
        arg2: impl Into<String>,
        arg3: impl Into<String>,
        arg4: impl Into<String>,
    ) {
        exec_command(CommandForBrowser::Log {
            kind,
            message: message.into(),
            arg2: arg2.into(),
            arg3: arg3.into(),
            arg4: arg4.into(),
        });
    }

    pub fn timezone_offset(&self) -> i32 {
        let response = exec_command(CommandForBrowser::TimezoneOffset);

        let response = decode_json::<browser_response::TimezoneOffset>(response);
        match response {
            Ok(response) => {
                // Return in seconds to be compatible with chrono
                // Opposite as JS returns the offset backwards
                response.value * -60
            }
            Err(err) => {
                log::error!("api.timezone_offset -> incorrect result = {err}");

                0
            }
        }
    }

    pub fn history_back(&self) {
        exec_command(CommandForBrowser::HistoryBack);
    }

    pub fn get_random(&self, min: u32, max: u32) -> u32 {
        let response = exec_command(CommandForBrowser::GetRandom { min, max });

        let response = decode_json::<browser_response::GetRandom>(response);
        match response {
            Ok(response) => {
                // Return in seconds to be compatible with chrono
                // Opposite as JS returns the offset backwards
                response.value
            }
            Err(err) => {
                log::error!("get_random -> incorrect result = {err}");

                min
            }
        }
    }
}
