#[cfg(all(not(test), target_arch = "wasm32", target_os = "unknown"))]
mod inner {
    #[link(wasm_import_module = "mod")]
    extern "C" {
        pub fn panic_message(long_ptr: u64);
        pub fn dom_access(long_ptr: u64) -> u64;
    }
}

#[cfg(all(not(test), target_arch = "wasm32", target_os = "unknown"))]
pub mod safe_wrappers {
    use super::inner::*;
    use crate::LongPtr;

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
        command::{decode_json, response_browser, CommandForBrowser},
        driver_module::api::api_arguments,
        JsJson, JsJsonSerialize, JsValue, LongPtr,
    };

    pub fn safe_panic_message(_long_ptr: LongPtr) {}

    pub fn safe_dom_access(long_ptr: LongPtr) -> LongPtr {
        let value = api_arguments().get_by_long_ptr(long_ptr);

        if let JsValue::Json(json) = value {
            let command = decode_json::<CommandForBrowser>(json);

            let response = match command {
                CommandForBrowser::FetchCacheGet => {
                    response_browser::FetchCacheGet { data: None }.to_json()
                }
                CommandForBrowser::FetchExec {
                    request: _,
                    callback: _,
                } => JsJson::Null,
                CommandForBrowser::SetStatus { status: _ } => JsJson::Null,
                CommandForBrowser::IsBrowser => {
                    let response = response_browser::IsBrowser { value: false };
                    response.to_json()
                }
                CommandForBrowser::GetDateNow => {
                    let response = response_browser::GetDateNow { value: 0 };
                    response.to_json()
                }
            };

            return JsValue::Json(response).to_ptr_long();
        }

        LongPtr::from(0)
    }
}
