use std::{rc::Rc, future::Future, pin::Pin, collections::HashMap};

use crate::{
    InstantType,
    DropResource,
    struct_mut::ValueMut,
    transaction,
    get_driver,
    FetchResult,
    FutureBox,
    FetchMethod,
    WebsocketMessage,
    WebsocketConnection,
    driver_module::{
        js_value::JsValue,
        utils::json::JsonMapBuilder
    }
};

use super::{
    panic_message::PanicMessage,
    api_dom_access::DomAccess,
    arguments::Arguments,
    callbacks::CallbackStore
};

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

#[derive(Clone)]
pub struct ApiImport {
    pub panic_message: PanicMessage,
    pub fn_dom_access: fn(ptr: u32, size: u32) -> u32,

    pub(crate) arguments: Arguments,
    pub(crate) callback_store: CallbackStore,
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
            arguments: Arguments::default(),
            callback_store: CallbackStore::new(),
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

    pub fn interval_set<F: Fn() + 'static>(&self, duration: u32, callback: F) -> DropResource {
        let (callback_id, drop_callback) = self.callback_store.register(move |_| {
            callback();
            JsValue::Undefined
        });

        let result = self.dom_access()
            .api()
            .get("interval")
            .call("interval_set", vec!(
                JsValue::U32(duration),
                JsValue::U64(callback_id.as_u64()),
            ))
            .fetch();

        let timer_id = if let JsValue::I32(timer_id) = result {
            timer_id
        } else {
            log::error!("interval_set -> expected i32 -> result={result:?}");
            0
        };

        let api = self.clone();

        DropResource::new(move || {
            api.dom_access()
                .api()
                .get("interval")
                .call("interval_clear", vec!(
                    JsValue::I32(timer_id),
                ))
                .exec();

            drop_callback.off();
        })
    }

    pub fn timeout_set<F: Fn() + 'static>(&self, duration: u32, callback: F) -> DropResource {
        let (callback_id, drop_callback) = self.callback_store.register(move |_| {
            callback();
            JsValue::Undefined
        });

        let result = self.dom_access()
            .api()
            .get("interval")
            .call("timeout_set", vec!(
                JsValue::U32(duration),
                JsValue::U64(callback_id.as_u64()),
            ))
            .fetch();

        let timer_id = if let JsValue::I32(timer_id) = result {
            timer_id
        } else {
            log::error!("timeout_set -> expected u32 -> result={result:?}");
            0
        };

        let api = self.clone();

        DropResource::new(move || {
            api.dom_access()
                .api()
                .get("interval")
                .call("interval_clear", vec!(
                    JsValue::I32(timer_id),
                ))
                .exec();

            drop_callback.off();
        })
    }

    pub fn set_timeout_and_detach<F: Fn() + 'static>(&self, duration: u32, callback: F) {
        let drop_box: Rc<ValueMut<Option<DropResource>>> = Rc::new(ValueMut::new(None));

        let callback_with_drop = {
            let drop_box = drop_box.clone();

            move || {
                callback();
                drop_box.set(None);
            }
        };

        let drop = self.timeout_set(duration, callback_with_drop);
        drop_box.set(Some(drop));
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

    pub fn get_hash_location(&self) -> String {
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

    pub fn push_hash_location(&self, new_hash: &str) {
        self.dom_access()
            .api()
            .get("hashRouter")
            .call("push", vec!(
                JsValue::str(new_hash)
            ))
            .exec();
    }

    pub fn on_hash_route_change<F: Fn(String) + 'static>(&self, callback: F) -> DropResource {
        let (callback_id, drop_callback) = self.callback_store.register(move |data| {
            let new_hash = if let JsValue::String(new_hash) = data {
                new_hash
            } else {
                log::error!("on_hash_route_change -> string was expected -> {data:?}");
                String::from("")
            };

            transaction(|_| {
                callback(new_hash);
            });

            JsValue::Undefined
        });

        self.dom_access()
            .api()
            .get("hashRouter")
            .call("add", vec!(
                JsValue::U64(callback_id.as_u64()),
            ))
            .exec();
            

        let api = self.clone();

        DropResource::new(move || {
            api.dom_access()
                .api()
                .get("hashRouter")
                .call("remove", vec!(
                    JsValue::U64(callback_id.as_u64()),
                ))
                .exec();
                
            drop_callback.off();
        })
    }

    pub fn fetch(
        &self,
        method: FetchMethod,
        url: String,
        headers: Option<HashMap<String, String>>,
        body: Option<String>,
    ) -> Pin<Box<dyn Future<Output = FetchResult> + 'static>> {
        let (sender, receiver) = FutureBox::new();

        let callback_id = self.callback_store.register_once(move |params| {
            let params = params
                .convert(|mut params| {
                    let success = params.get_bool("success")?;
                    let status = params.get_u32("status")?;
                    let response = params.get_string("response")?;
                    params.expect_no_more()?;
                    Ok((success, status, response))
                });
    
            match params {
                Ok((success, status, response)) => {
                    get_driver().transaction(|_| {
                        let response = match success {
                            true => Ok((status, response)),
                            false => Err(response),
                        };
                        sender.publish(response);
                    });
                },
                Err(error) => {
                    log::error!("export_fetch_callback -> params decode error -> {error}");
                }
            }

            JsValue::Undefined
        });

        let headers = {
            let mut headers_builder = JsonMapBuilder::new();

            if let Some(headers) = headers {
                for (key, value) in headers.into_iter() {
                    headers_builder.set_string(&key, &value);
                }
            }

            headers_builder.build()
        };

        self.dom_access()
            .api()
            .get("fetch")
            .call("fetch_send_request", vec!(
                JsValue::U64(callback_id.as_u64()),
                JsValue::String(method.to_str()),
                JsValue::String(url),
                JsValue::String(headers),
                match body {
                    Some(body) => JsValue::String(body),
                    None => JsValue::Null,
                },
            ))
            .exec();
        
        Box::pin(receiver)
    }

    #[must_use]
    pub fn websocket<F: Fn(WebsocketMessage) + 'static>(&self, host: impl Into<String>, callback: F) -> DropResource {
        let host: String = host.into();

        let api = self.clone();

        let (callback_id, drop_callback) = self.callback_store.register_with_id(move |callback_id, data| {
            if let JsValue::True = data {
                let connection = WebsocketConnection::new(api.clone(), callback_id);
                let connection = WebsocketMessage::Connection(connection);
                callback(connection);
                return JsValue::Undefined;
            }

            if let JsValue::String(message) = data {
                callback(WebsocketMessage::Message(message));
                return JsValue::Undefined;
            }

            if let JsValue::False = data {
                callback(WebsocketMessage::Close);
                return JsValue::Undefined;
            }

            log::error!("websocket - unsupported message type received");
            JsValue::Undefined
        });

        self.websocket_register_callback(host.as_str(), callback_id.as_u64());

        DropResource::new({
            let api = self.clone();

            move || {
                api.websocket_unregister_callback(callback_id.as_u64());
                drop_callback.off();
            }
        })
    }

    fn websocket_register_callback(&self, host: &str, callback_id: u64) {
        self.dom_access()
            .api()
            .get("websocket")
            .call("websocket_register_callback", vec!(
                JsValue::String(host.to_string()),
                JsValue::U64(callback_id)
            ))
            .exec();
    }

    fn websocket_unregister_callback(&self, callback_id: u64) {
        self.dom_access()
            .api()
            .get("websocket")
            .call("websocket_unregister_callback", vec!(
                JsValue::U64(callback_id)
            ))
            .exec();
    }

    pub fn websocket_send_message(&self, callback_id: u64, message: &str) {
        self.dom_access()
            .api()
            .get("websocket")
            .call("websocket_send_message", vec!(
                JsValue::U64(callback_id),
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

    pub fn get_random(&self, min: u32, max: u32) -> u32 {
        let result = self.dom_access()
            .api()
            .call("getRandom", vec!(
                JsValue::U32(min),
                JsValue::U32(max)
            ))
            .fetch();

        if let JsValue::I32(result) = result {
            result as u32
        } else {
            self.panic_message.show(format!("api.get_random -> incorrect result {result:?}"));
            min
        }
    }
}

