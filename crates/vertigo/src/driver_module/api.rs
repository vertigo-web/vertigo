use crate::InstantType;

use super::{js_value::js_value_builder::JsValueBuilder, js_value::{Arguments, js_value_struct::JsValue}};

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

pub struct DomAccess {
    panic_message: PanicMessage,
    arguments: Arguments,
    fn_dom_access: fn(ptr: u32, size: u32) -> u32,
    builder: JsValueBuilder,
}

impl DomAccess {
    pub fn new(panic_message: PanicMessage, arguments: Arguments, fn_dom_access: fn(ptr: u32, size: u32) -> u32) -> DomAccess {
        DomAccess {
            panic_message,
            fn_dom_access,
            arguments,
            builder: JsValueBuilder::new(),
        }
    }

    pub fn element(mut self, dom_id: u64) -> Self {
        let value = JsValueBuilder::new()
            .str("get")
            .u64(dom_id)
            .get();
        
        self.builder.value_push(value);
        self
    }

    pub fn get(mut self, name: impl Into<String>) -> Self {
        let value = JsValueBuilder::new()
            .str("get")
            .string(name)
            .get();
        
            self.builder.value_push(value);
        self
    }

    pub fn set(mut self, name: impl Into<String>, value: JsValue) -> Self {
        let value = JsValueBuilder::new()
            .str("set")
            .string(name)
            .value(value)
            .get();
        
            self.builder.value_push(value);
        self
    }

    pub fn call(mut self, name: impl Into<String>, params: Vec<JsValue>) -> Self {
        let value = JsValueBuilder::new()
            .str("call")
            .string(name)
            .extend(params);

        self.builder.value_push(value.get());
        self
    }

    pub fn get_props(mut self, props: &[&str]) -> Self {
        let value = JsValueBuilder::new()
            .str("get_props")
            .list(|mut list| {
                for prop in props {
                    list.str_push(prop);
                }

                list
            });
            
        self.builder.value_push(value.get());
        self
    }

    pub fn exec(self) {
        let panic_message = self.panic_message;

        let result = self.fetch();

        if let JsValue::Undefined = result {
            //ok
        } else {
            let message = format!("Expected undefined dump={result:?}");
            panic_message.show(message);
        }
    }

    pub fn fetch(self) -> JsValue {
        let memory = self.builder.build();
        let (ptr, size) = memory.get_ptr_and_size();

        let result_ptr = (self.fn_dom_access)(ptr, size);
        self.arguments.get_by_ptr(result_ptr).unwrap_or(JsValue::Undefined)
    }
}

pub struct ApiImport {
    pub panic_message: PanicMessage,
    js_call: fn(params: u32, size: u32) -> u32,

    pub interval_set: fn(duration: u32, callback_id: u32) -> u32,
    pub interval_clear: fn(timer_id: u32),
    pub timeout_set: fn(duration: u32, callback_id: u32) -> u32,
    pub timeout_clear: fn(timer_id: u32),

    pub fn_dom_access: fn(ptr: u32, size: u32) -> u32,

    pub arguments: Arguments,
}

impl ApiImport {

    pub fn new(
        panic_message: fn(ptr: u32, size: u32),
        js_call: fn(params: u32, size: u32) -> u32,

        interval_set: fn(duration: u32, callback_id: u32) -> u32,
        interval_clear: fn(timer_id: u32),
        timeout_set: fn(duration: u32, callback_id: u32) -> u32,
        timeout_clear: fn(timer_id: u32),

        fn_dom_access: fn(ptr: u32, size: u32) -> u32,
    
    ) -> ApiImport {
        let panic_message = PanicMessage::new(panic_message);

        ApiImport {
            panic_message,
            js_call,
            interval_set,
            interval_clear,
            timeout_set,
            timeout_clear,

            fn_dom_access,

            arguments: Arguments::new(),
        }
    }

    pub fn show_panic_message(&self, message: String) {
        self.panic_message.show(message);
    }

    fn js_call(&self, params: JsValueBuilder) -> Option<JsValue> {
        let params_memory = params.build();
        let (ptr, size) = params_memory.get_ptr_and_size();

        let result_ptr = (self.js_call)(ptr, size);
        self.arguments.get_by_ptr(result_ptr)
    }

    fn console_4(&self, kind: ConsoleLogLevel, arg1: &str, arg2: &str, arg3: &str, arg4: &str) {
        self.dom_access()
            .get("window")
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
        let params = JsValueBuilder::new()
            .str("module")
            .str("cookie")
            .str("get")
            .string(cname);

        let result = self.js_call(params);

        result
            .unwrap_or_default()
            .convert::<String, _>(|mut params| {
                let first = params.get_string("first")?;
                params.expect_no_more()?;
                Ok(first)
            })
            .unwrap_or_else(|error| {
                log::error!("cookie_get -> params decode error -> {error}");
                String::from("")
            })
    }

    pub fn cookie_set(&self, cname: &str, cvalue: &str, expires_in: u64) {
        let params = JsValueBuilder::new()
            .str("module")
            .str("cookie")
            .str("set")
            .string(cname)
            .string(cvalue)
            .u64(expires_in);

        let _ = self.js_call(params);
    }

    pub fn interval_set(&self, duration: u32, callback_id: u32) -> u32 {
        let interval_set = self.interval_set;
        interval_set(duration, callback_id)
    }

    pub fn interval_clear(&self, timer_id: u32) {
        let interval_clear = self.interval_clear;
        interval_clear(timer_id)
    }

    pub fn timeout_set(&self, duration: u32, callback_id: u32) -> u32 {
        let timeout_set = self.timeout_set;
        timeout_set(duration, callback_id)
    }

    #[allow(dead_code)]
    pub fn timeout_clear(&self, timer_id: u32) {
        let timeout_clear = self.timeout_clear;
        timeout_clear(timer_id)
    }

    pub fn instant_now(&self) -> InstantType {
        let result = self.dom_access()
            .get("window")
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
        let params = JsValueBuilder::new()
            .str("module")
            .str("hashrouter")
            .str("get");

        let result = self.js_call(params);

        result
            .unwrap_or_default()
            .convert::<String, _>(|mut params| {
                let first = params.get_string("first")?;
                params.expect_no_more()?;
                Ok(first)
            })
            .unwrap_or_else(|error| {
                log::error!("hashrouter_get_hash_location -> params decode error -> {error}");
                String::from("")
            })
    }

    pub fn hashrouter_push_hash_location(&self, new_hash: &str) {
        let params = JsValueBuilder::new()
            .str("module")
            .str("hashrouter")
            .str("push")
            .string(new_hash);

        let _ = self.js_call(params);
    }

    pub fn fetch_send_request(
        &self,
        request_id: u32,
        method: String,
        url: String,
        headers: String,
        body: Option<String>,
    ) {
        let params = JsValueBuilder::new()
            .str("module")
            .str("fetch")
            .str("send")
            .u32(request_id)
            .string(method)
            .string(url)
            .string(headers)
            .string_option(body);

        let _ = self.js_call(params);
    }

    pub fn websocket_register_callback(&self, host: &str, callback_id: u32) {
        let params = JsValueBuilder::new()
            .str("module")
            .str("websocket")
            .str("register_callback")
            .string(host)
            .u32(callback_id);

        let _ = self.js_call(params);
    }

    pub fn websocket_unregister_callback(&self, callback_id: u32) {
        let params = JsValueBuilder::new()
            .str("module")
            .str("websocket")
            .str("unregister_callback")
            .u32(callback_id);

        let _ = self.js_call(params);
    }

    pub fn websocket_send_message(&self, callback_id: u32, message: &str) {
        let params = JsValueBuilder::new()
            .str("module")
            .str("websocket")
            .str("send_message")
            .u32(callback_id)
            .string(message);

        let _ = self.js_call(params);
    }

    pub fn dom_bulk_update(&self, value: &str) {
        let params = JsValueBuilder::new()
            .str("module")
            .str("dom")
            .str("dom_bulk_update")
            .string(value);

        let _ = self.js_call(params);
    }

    pub fn dom_access(&self) -> DomAccess {
        DomAccess::new(
            self.panic_message,
            self.arguments.clone(),
            self.fn_dom_access
        )
    }
}

