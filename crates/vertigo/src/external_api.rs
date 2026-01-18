#[cfg(all(not(test), target_arch = "wasm32", target_os = "unknown"))]
mod inner {
    #[link(wasm_import_module = "mod")]
    unsafe extern "C" {
        pub fn panic_message(long_ptr: u64);
        pub fn dom_access(long_ptr: u64) -> u64;
    }
}

#[cfg(all(not(test), target_arch = "wasm32", target_os = "unknown"))]
pub mod safe_wrappers {
    use crate::dev::LongPtr;

    use super::inner::*;

    pub fn safe_panic_message(long_ptr: LongPtr) {
        let long_ptr = long_ptr.get_long_ptr();
        unsafe { panic_message(long_ptr) }
    }

    pub fn safe_dom_access(long_ptr: LongPtr) -> LongPtr {
        let long_ptr = long_ptr.get_long_ptr();
        let result = unsafe { dom_access(long_ptr) };
        LongPtr::from(result)
    }
}

#[cfg(any(test, not(target_arch = "wasm32"), not(target_os = "unknown")))]
pub mod safe_wrappers {
    use crate::{
        JsJson, JsJsonSerialize,
        dev::{
            LongPtr,
            command::{CommandForBrowser, browser_response, decode_json},
        },
        driver_module::api::api_arguments,
    };

    pub fn safe_panic_message(_long_ptr: LongPtr) {}

    pub fn safe_dom_access(long_ptr: LongPtr) -> LongPtr {
        let json = api_arguments().get_by_long_ptr(long_ptr);

        let command_res = decode_json::<CommandForBrowser>(json);

        match command_res {
            Ok(command) => {
                let response = match command {
                    CommandForBrowser::FetchCacheGet => {
                        browser_response::FetchCacheGet { data: None }.to_json()
                    }
                    CommandForBrowser::IsBrowser => {
                        let response = browser_response::IsBrowser { value: false };
                        response.to_json()
                    }
                    CommandForBrowser::GetDateNow => {
                        let response = browser_response::GetDateNow { value: 0 };
                        response.to_json()
                    }
                    CommandForBrowser::LocationGet { target: _ } => {
                        browser_response::LocationGet { value: "".into() }.to_json()
                    }
                    CommandForBrowser::CookieGet { name: _ } => {
                        browser_response::CookieGet { value: "".into() }.to_json()
                    }
                    CommandForBrowser::CookieJsonGet { name: _ } => {
                        browser_response::CookieJsonGet {
                            value: JsJson::Null,
                        }
                        .to_json()
                    }
                    CommandForBrowser::GetEnv { name: _ } => {
                        browser_response::GetEnv { value: None }.to_json()
                    }
                    CommandForBrowser::TimezoneOffset => {
                        browser_response::TimezoneOffset { value: 0 }.to_json()
                    }
                    CommandForBrowser::GetRandom { min, max: _ } => {
                        browser_response::GetRandom { value: min }.to_json()
                    }
                    CommandForBrowser::FetchExec { .. }
                    | CommandForBrowser::SetStatus { .. }
                    | CommandForBrowser::WebsocketRegister { .. }
                    | CommandForBrowser::WebsocketUnregister { .. }
                    | CommandForBrowser::WebsocketSendMessage { .. }
                    | CommandForBrowser::TimerSet { .. }
                    | CommandForBrowser::TimerClear { .. }
                    | CommandForBrowser::LocationCallback { .. }
                    | CommandForBrowser::LocationSet { .. }
                    | CommandForBrowser::CookieSet { .. }
                    | CommandForBrowser::CookieJsonSet { .. }
                    | CommandForBrowser::Log { .. }
                    | CommandForBrowser::HistoryBack
                    | CommandForBrowser::JsApiCall { .. }
                    | CommandForBrowser::DomBulkUpdate { .. } => JsJson::Null,
                };

                return response.to_ptr_long();
            }
            Err(err) => {
                log::error!("safe_dom_access -> decode error = {err}");
            }
        }

        LongPtr::from(0)
    }
}
