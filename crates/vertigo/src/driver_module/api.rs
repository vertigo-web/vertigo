use crate::InstantType;

use super::{js_value::{Arguments, js_value_struct::JsValue}, DomAccess};

enum ConsoleLogLevel {
    Debug,
    Info,
    Log,
    Warn,
    Error
}

impl ConsoleLogLevel {
    pub fn get_str(&self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Log => "log",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Copy)]
pub struct PanicMessage {
    panic_message: fn(ptr: u32, size: u32),
}

impl PanicMessage {
    pub fn new(panic_message: fn(ptr: u32, size: u32)) -> PanicMessage {
        PanicMessage {
            panic_message
        }
    }

    pub fn show(&self, message: impl Into<String>) {
        let message = message.into();
        let ptr = message.as_ptr() as u32;
        let len = message.len() as u32;
        (self.panic_message)(ptr, len);
    }
}

pub struct ApiImport {
    pub panic_message: PanicMessage,
    pub fn_dom_access: fn(ptr: u32, size: u32) -> u32,

    pub arguments: Arguments,
}

impl ApiImport {

    pub fn new(
        panic_message: fn(ptr: u32, size: u32),
        fn_dom_access: fn(ptr: u32, size: u32) -> u32,
    
    ) -> ApiImport {
        let panic_message = PanicMessage::new(panic_message);

        ApiImport {
            panic_message,
            fn_dom_access,
            arguments: Arguments::new(),
        }
    }

    pub fn show_panic_message(&self, message: String) {
        self.panic_message.show(message);
    }

    fn console_4(&self, kind: ConsoleLogLevel, arg1: &str, arg2: &str, arg3: &str, arg4: &str) {
        self.dom_access()
            .root("window")
            .get("console")
            .call(kind.get_str(), vec!(
                JsValue::str(arg1),
                JsValue::str(arg2),
                JsValue::str(arg3),
                JsValue::str(arg4),
            ))
            .exec();
    }

    pub fn console_debug_4(&self, arg1: &str, arg2: &str, arg3: &str, arg4: &str) {
        self.console_4(ConsoleLogLevel::Debug, arg1, arg2, arg3, arg4)
    }

    pub fn console_log_4(&self, arg1: &str, arg2: &str, arg3: &str, arg4: &str) {
        self.console_4(ConsoleLogLevel::Log, arg1, arg2, arg3, arg4)
    }

    pub fn console_info_4(&self, arg1: &str, arg2: &str, arg3: &str, arg4: &str) {
        self.console_4(ConsoleLogLevel::Info, arg1, arg2, arg3, arg4)
    }

    pub fn console_warn_4(&self, arg1: &str, arg2: &str, arg3: &str, arg4: &str) {
        self.console_4(ConsoleLogLevel::Warn, arg1, arg2, arg3, arg4)
    }

    pub fn console_error_4(&self, arg1: &str, arg2: &str, arg3: &str, arg4: &str) {
        self.console_4(ConsoleLogLevel::Error, arg1, arg2, arg3, arg4)
    }

    pub fn cookie_get(&self, cname: &str) -> String {
        let result = self.dom_access()
            .api()
            .get("cookie")
            .call("get", vec!(
                JsValue::str(cname)
            ))
            .fetch();

        if let JsValue::String(value) = result {
            value
        } else {
            log::error!("cookie_get -> params decode error -> result={result:?}");
            String::from("")
        }
    }

    pub fn cookie_set(&self, cname: &str, cvalue: &str, expires_in: u64) {
        self.dom_access()
            .api()
            .get("cookie")
            .call("set", vec!(
                JsValue::str(cname),
                JsValue::str(cvalue),
                JsValue::U64(expires_in)
            ))
            .exec();
    }

    pub fn interval_set(&self, duration: u32, callback_id: u32) -> i32 {
        let result = self.dom_access()
            .api()
            .get("interval")
            .call("interval_set", vec!(
                JsValue::U32(duration),
                JsValue::U32(callback_id),
            ))
            .fetch();
        
        if let JsValue::I32(timer_id) = result {
            timer_id
        } else {
            log::error!("interval_set -> expected u32 -> result={result:?}");
            0
        }
    }

    pub fn interval_clear(&self, timer_id: i32) {
        self.dom_access()
            .api()
            .get("interval")
            .call("interval_clear", vec!(
                JsValue::I32(timer_id),
            ))
            .exec();  
    }

    pub fn timeout_set(&self, duration: u32, callback_id: u32) -> i32 {
        let result = self.dom_access()
            .api()
            .get("interval")
            .call("timeout_set", vec!(
                JsValue::U32(duration),
                JsValue::U32(callback_id),
            ))
            .fetch();
        
        if let JsValue::I32(timer_id) = result {
            timer_id
        } else {
            log::error!("timeout_set -> expected u32 -> result={result:?}");
            0
        }
    }

    #[allow(dead_code)]
    pub fn timeout_clear(&self, timer_id: i32) {
        self.dom_access()
            .api()
            .get("interval")
            .call("interval_clear", vec!(
                JsValue::I32(timer_id),
            ))
            .exec();
    }

    pub fn instant_now(&self) -> InstantType {
        let result = self.dom_access()
            .root("window")
            .get("Date")
            .call("now", vec!())
            .fetch();

        if let JsValue::I64(time) = result {
            time as u64 as InstantType
        } else {
            self.panic_message.show(format!("api.instant_now -> incorrect result {result:?}"));
            0_u64
        }
    }

    pub fn hashrouter_get_hash_location(&self) -> String {
        let result = self.dom_access()
            .api()
            .get("hashRouter")
            .call("get", Vec::new())
            .fetch();

        if let JsValue::String(value) = result {
            value
        } else {
            log::error!("hashRouter -> params decode error -> result={result:?}");
            String::from("")
        }
    }

    pub fn hashrouter_push_hash_location(&self, new_hash: &str) {
        self.dom_access()
            .api()
            .get("hashRouter")
            .call("push", vec!(
                JsValue::str(new_hash)
            ))
            .exec();
    }

    pub fn fetch_send_request(
        &self,
        request_id: u32,
        method: String,
        url: String,
        headers: String,
        body: Option<String>,
    ) {
        self.dom_access()
            .api()
            .get("fetch")
            .call("fetch_send_request", vec!(
                JsValue::U32(request_id),
                JsValue::String(method),
                JsValue::String(url),
                JsValue::String(headers),
                match body {
                    Some(body) => JsValue::String(body),
                    None => JsValue::Null,
                },
            ))
            .exec();
    }

    pub fn websocket_register_callback(&self, host: &str, callback_id: u32) {
        self.dom_access()
            .api()
            .get("websocket")
            .call("websocket_register_callback", vec!(
                JsValue::String(host.to_string()),
                JsValue::U32(callback_id)
            ))
            .exec();
    }

    pub fn websocket_unregister_callback(&self, callback_id: u32) {
        self.dom_access()
            .api()
            .get("websocket")
            .call("websocket_unregister_callback", vec!(
                JsValue::U32(callback_id)
            ))
            .exec();
    }

    pub fn websocket_send_message(&self, callback_id: u32, message: &str) {
        self.dom_access()
            .api()
            .get("websocket")
            .call("websocket_send_message", vec!(
                JsValue::U32(callback_id),
                JsValue::String(message.to_string())
            ))
            .exec();
    }

    pub fn dom_bulk_update(&self, value: &str) {
        self.dom_access()
            .api()
            .get("dom")
            .call("dom_bulk_update", vec!(
                JsValue::str(value)
            ))
            .exec();
    }

    pub fn dom_access(&self) -> DomAccess {
        DomAccess::new(
            self.panic_message,
            self.arguments.clone(),
            self.fn_dom_access
        )
    }
}

