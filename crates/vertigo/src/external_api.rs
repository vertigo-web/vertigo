#[cfg(all(not(test), target_arch = "wasm32", target_os = "unknown"))]
mod inner {
    #[link(wasm_import_module = "mod")]
    unsafe extern "C" {
        pub fn panic_message(long_ptr: u64);
        pub fn dom_access(long_ptr: u64) -> u64;
        pub fn dom_bulk_update(long_ptr: u64);
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

    /// A batch of DOM commands, read where it is while the call lasts.
    pub fn safe_dom_bulk_update(commands: &[u8]) {
        let long_ptr = LongPtr::new(commands.as_ptr() as u32, commands.len() as u32);
        unsafe { dom_bulk_update(long_ptr.get_long_ptr()) }
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

    pub fn safe_dom_bulk_update(_commands: &[u8]) {}

    #[cfg(test)]
    thread_local! {
        /// Env the tests pretend the server passed to the app (`Driver::env`).
        pub static TEST_ENV: std::cell::RefCell<std::collections::HashMap<String, String>> =
            Default::default();

        /// Responses the tests pretend the server fetched while rendering the page
        /// (`data-fetch-cache`, an encoded `SsrFetchCache`).
        pub static TEST_FETCH_CACHE: std::cell::RefCell<Option<String>> = Default::default();

        /// Whether the tests pretend to run in the browser (`Driver::is_browser`), not on the
        /// server as by default.
        pub static TEST_BROWSER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }

    #[cfg(test)]
    fn is_browser() -> bool {
        TEST_BROWSER.get()
    }

    #[cfg(not(test))]
    fn is_browser() -> bool {
        false
    }

    #[cfg(test)]
    fn env(name: &str) -> Option<String> {
        TEST_ENV.with(|env| env.borrow().get(name).cloned())
    }

    #[cfg(not(test))]
    fn env(_name: &str) -> Option<String> {
        None
    }

    #[cfg(test)]
    fn fetch_cache() -> Option<String> {
        TEST_FETCH_CACHE.with(|cache| cache.borrow().clone())
    }

    #[cfg(not(test))]
    fn fetch_cache() -> Option<String> {
        None
    }

    pub fn safe_dom_access(long_ptr: LongPtr) -> LongPtr {
        let json = api_arguments().get_by_long_ptr(long_ptr);

        let command_res = decode_json::<CommandForBrowser>(json);

        match command_res {
            Ok(command) => {
                let response = match command {
                    CommandForBrowser::FetchCacheGet => browser_response::FetchCacheGet {
                        data: fetch_cache(),
                    }
                    .to_json(),
                    CommandForBrowser::IsBrowser => {
                        let response = browser_response::IsBrowser {
                            value: is_browser(),
                        };
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
                    CommandForBrowser::GetEnv { name } => {
                        browser_response::GetEnv { value: env(&name) }.to_json()
                    }
                    CommandForBrowser::TimezoneOffset => {
                        browser_response::TimezoneOffset { value: 0 }.to_json()
                    }
                    CommandForBrowser::Languages => {
                        browser_response::Languages { value: Vec::new() }.to_json()
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
                    | CommandForBrowser::AnimationFrameStart
                    | CommandForBrowser::AnimationFrameStop
                    | CommandForBrowser::LocationCallback { .. }
                    | CommandForBrowser::LocationSet { .. }
                    | CommandForBrowser::CookieSet { .. }
                    | CommandForBrowser::CookieJsonSet { .. }
                    | CommandForBrowser::Log { .. }
                    | CommandForBrowser::HistoryBack
                    | CommandForBrowser::ImagePreload { .. }
                    | CommandForBrowser::PointerLock { .. }
                    | CommandForBrowser::Fullscreen { .. }
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
