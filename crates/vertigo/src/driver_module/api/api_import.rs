use std::{collections::HashMap, future::Future, pin::Pin, rc::Rc};

use crate::{
    driver_module::{event_emitter::EventEmitter, js_value::JsValue},
    fetch::request_builder::RequestBody,
    get_driver,
    struct_mut::ValueMut,
    transaction, DropResource, FetchMethod, FetchResult, FutureBox, InstantType, JsJson,
    JsJsonObjectBuilder, WebsocketConnection, WebsocketMessage,
};

use super::{
    api_dom_access::DomAccess, arguments::Arguments, callbacks::CallbackStore,
    panic_message::PanicMessage,
};

enum ConsoleLogLevel {
    Debug,
    Info,
    Log,
    Warn,
    Error,
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

    pub on_fetch_start: EventEmitter<()>,
    pub on_fetch_stop: EventEmitter<()>,
}

impl Default for ApiImport {
    fn default() -> Self {
        use super::external_api::api::safe_wrappers::{
            safe_dom_access as fn_dom_access, safe_panic_message as panic_message,
        };

        let panic_message = PanicMessage::new(panic_message);

        ApiImport {
            panic_message,
            fn_dom_access,
            arguments: Arguments::default(),
            callback_store: CallbackStore::new(),
            on_fetch_start: EventEmitter::default(),
            on_fetch_stop: EventEmitter::default(),
        }
    }
}

impl ApiImport {
    pub fn show_panic_message(&self, message: String) {
        self.panic_message.show(message);
    }

    fn console_4(&self, kind: ConsoleLogLevel, arg1: &str, arg2: &str, arg3: &str, arg4: &str) {
        self.dom_access()
            .root("window")
            .get("console")
            .call(
                kind.get_str(),
                vec![
                    JsValue::str(arg1),
                    JsValue::str(arg2),
                    JsValue::str(arg3),
                    JsValue::str(arg4),
                ],
            )
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
        let result = self
            .dom_access()
            .api()
            .get("cookie")
            .call("get", vec![JsValue::str(cname)])
            .fetch();

        if let JsValue::String(value) = result {
            value
        } else {
            log::error!("cookie_get -> params decode error -> result={result:?}");
            String::from("")
        }
    }

    pub fn cookie_get_json(&self, cname: &str) -> JsJson {
        if self.is_browser() {
            let result = self
                .dom_access()
                .api()
                .get("cookie")
                .call("get_json", vec![JsValue::str(cname)])
                .fetch();

            if result != JsValue::Null {
                if let JsValue::Json(value) = result {
                    return value;
                }
                log::error!("cookie_get_json -> params decode error -> result={result:?}");
            }
        }
        JsJson::Null
    }

    pub fn cookie_set(&self, cname: &str, cvalue: &str, expires_in: u64) {
        if self.is_browser() {
            self.dom_access()
                .api()
                .get("cookie")
                .call(
                    "set",
                    vec![
                        JsValue::str(cname),
                        JsValue::str(cvalue),
                        JsValue::U64(expires_in),
                    ],
                )
                .exec();
        } else {
            log::warn!("Can't set cookie on server side");
        }
    }

    pub fn cookie_set_json(&self, cname: &str, cvalue: JsJson, expires_in: u64) {
        self.dom_access()
            .api()
            .get("cookie")
            .call(
                "set_json",
                vec![
                    JsValue::str(cname),
                    JsValue::Json(cvalue),
                    JsValue::U64(expires_in),
                ],
            )
            .exec();
    }

    pub fn interval_set<F: Fn() + 'static>(&self, duration: u32, callback: F) -> DropResource {
        let (callback_id, drop_callback) = self.callback_store.register(move |_| {
            callback();
            JsValue::Undefined
        });

        let result = self
            .dom_access()
            .api()
            .get("interval")
            .call(
                "interval_set",
                vec![JsValue::U32(duration), JsValue::U64(callback_id.as_u64())],
            )
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
                .call("interval_clear", vec![JsValue::I32(timer_id)])
                .exec();

            drop_callback.off();
        })
    }

    pub fn timeout_set<F: Fn() + 'static>(&self, duration: u32, callback: F) -> DropResource {
        let (callback_id, drop_callback) = self.callback_store.register(move |_| {
            callback();
            JsValue::Undefined
        });

        let result = self
            .dom_access()
            .api()
            .get("interval")
            .call(
                "timeout_set",
                vec![JsValue::U32(duration), JsValue::U64(callback_id.as_u64())],
            )
            .fetch();

        let timer_id = if let JsValue::I32(timer_id) = result {
            timer_id
        } else {
            log::error!("timeout_set -> expected i32 -> result={result:?}");
            0
        };

        let api = self.clone();

        DropResource::new(move || {
            api.dom_access()
                .api()
                .get("interval")
                .call("interval_clear", vec![JsValue::I32(timer_id)])
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
        self.utc_now() as InstantType
    }

    pub fn utc_now(&self) -> i64 {
        let result = self
            .dom_access()
            .root("window")
            .get("Date")
            .call("now", vec![])
            .fetch();

        match result {
            JsValue::I64(time) => time,
            JsValue::F64(time) => time as i64,
            _ => {
                self.panic_message
                    .show(format!("api.utc_now -> incorrect result {result:?}"));
                0_i64
            }
        }
    }

    pub fn timezone_offset(&self) -> i32 {
        let result = self
            .dom_access()
            .api()
            .call("getTimezoneOffset", vec![])
            .fetch();

        if let JsValue::I32(result) = result {
            // Return in seconds to be compatible with chrono
            // Opposite as JS returns the offset backwards
            result * -60
        } else {
            self.panic_message.show(format!(
                "api.timezone_offset -> incorrect result {result:?}"
            ));
            0
        }
    }

    pub fn history_back(&self) {
        self.dom_access()
            .root("window")
            .get("history")
            .call("back", Vec::new())
            .exec();
    }

    ///////////////////////////////////////////////////////////////////////////////////
    // hash router
    ///////////////////////////////////////////////////////////////////////////////////

    pub fn get_hash_location(&self) -> String {
        let result = self
            .dom_access()
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
            .call("push", vec![JsValue::str(new_hash)])
            .exec();
    }

    pub fn on_hash_change<F: Fn(String) + 'static>(&self, callback: F) -> DropResource {
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
            .call("add", vec![JsValue::U64(callback_id.as_u64())])
            .exec();

        let api = self.clone();

        DropResource::new(move || {
            api.dom_access()
                .api()
                .get("hashRouter")
                .call("remove", vec![JsValue::U64(callback_id.as_u64())])
                .exec();

            drop_callback.off();
        })
    }

    ///////////////////////////////////////////////////////////////////////////////////
    // history router
    ///////////////////////////////////////////////////////////////////////////////////

    pub fn get_history_location(&self) -> String {
        let result = self
            .dom_access()
            .api()
            .get("historyLocation")
            .call("get", Vec::new())
            .fetch();

        if let JsValue::String(value) = result {
            value
        } else {
            log::error!("historyLocation -> params decode error -> result={result:?}");
            String::from("")
        }
    }

    pub fn push_history_location(&self, new_path: &str) {
        self.dom_access()
            .api()
            .get("historyLocation")
            .call("push", vec![JsValue::str(new_path)])
            .exec();
    }

    pub fn replace_history_location(&self, new_hash: &str) {
        self.dom_access()
            .api()
            .get("historyLocation")
            .call("replace", vec![JsValue::str(new_hash)])
            .exec();
    }

    pub fn on_history_change<F: Fn(String) + 'static>(&self, callback: F) -> DropResource {
        let (callback_id, drop_callback) = self.callback_store.register(move |data| {
            let new_path = if let JsValue::String(new_path) = data {
                new_path
            } else {
                log::error!("on_history_change -> string was expected -> {data:?}");
                String::from("")
            };

            transaction(|_| {
                callback(new_path);
            });

            JsValue::Undefined
        });

        self.dom_access()
            .api()
            .get("historyLocation")
            .call("add", vec![JsValue::U64(callback_id.as_u64())])
            .exec();

        let api = self.clone();

        DropResource::new(move || {
            api.dom_access()
                .api()
                .get("historyLocation")
                .call("remove", vec![JsValue::U64(callback_id.as_u64())])
                .exec();

            drop_callback.off();
        })
    }

    ///////////////////////////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////////////////////////////////

    pub fn fetch(
        &self,
        method: FetchMethod,
        url: String,
        headers: Option<HashMap<String, String>>,
        body: Option<RequestBody>,
    ) -> Pin<Box<dyn Future<Output = FetchResult> + 'static>> {
        let (sender, receiver) = FutureBox::new();

        self.on_fetch_start.trigger(());

        let on_fetch_stop = self.on_fetch_stop.clone();

        let callback_id = self.callback_store.register_once(move |params| {
            let params = params.convert(|mut params| {
                let success = params.get_bool("success")?;
                let status = params.get_u32("status")?;
                let response = params.get_any("response")?;
                params.expect_no_more()?;

                if let JsValue::Json(json) = response {
                    return Ok((success, status, RequestBody::Json(json)));
                }

                if let JsValue::String(text) = response {
                    return Ok((success, status, RequestBody::Text(text)));
                }

                if let JsValue::Vec(buffer) = response {
                    return Ok((success, status, RequestBody::Binary(buffer)));
                }

                let name = response.typename();
                Err(format!(
                    "Expected json or string or vec<u8>, received={name}"
                ))
            });

            match params {
                Ok((success, status, response)) => {
                    get_driver().transaction(|_| {
                        let response = match success {
                            true => Ok((status, response)),
                            false => Err(format!("{response:#?}")),
                        };
                        sender.publish(response);
                        on_fetch_stop.trigger(());
                    });
                }
                Err(error) => {
                    log::error!("export_fetch_callback -> params decode error -> {error}");
                    on_fetch_stop.trigger(());
                }
            }

            JsValue::Undefined
        });

        let headers = {
            let mut headers_builder = JsJsonObjectBuilder::default();

            if let Some(headers) = headers {
                for (key, value) in headers.into_iter() {
                    headers_builder = headers_builder.insert(key, value);
                }
            }

            headers_builder.get()
        };

        self.dom_access()
            .api()
            .get("fetch")
            .call(
                "fetch_send_request",
                vec![
                    JsValue::U64(callback_id.as_u64()),
                    JsValue::String(method.to_str()),
                    JsValue::String(url),
                    JsValue::Json(headers),
                    match body {
                        Some(RequestBody::Text(body)) => JsValue::String(body),
                        Some(RequestBody::Json(json)) => JsValue::Json(json),
                        Some(RequestBody::Binary(bin)) => JsValue::Vec(bin),
                        None => JsValue::Undefined,
                    },
                ],
            )
            .exec();

        Box::pin(receiver)
    }

    #[must_use]
    pub fn websocket<F: Fn(WebsocketMessage) + 'static>(
        &self,
        host: impl Into<String>,
        callback: F,
    ) -> DropResource {
        let host: String = host.into();

        let api = self.clone();

        let (callback_id, drop_callback) =
            self.callback_store
                .register_with_id(move |callback_id, data| {
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
            .call(
                "websocket_register_callback",
                vec![JsValue::String(host.to_string()), JsValue::U64(callback_id)],
            )
            .exec();
    }

    fn websocket_unregister_callback(&self, callback_id: u64) {
        self.dom_access()
            .api()
            .get("websocket")
            .call(
                "websocket_unregister_callback",
                vec![JsValue::U64(callback_id)],
            )
            .exec();
    }

    pub fn websocket_send_message(&self, callback_id: u64, message: &str) {
        self.dom_access()
            .api()
            .get("websocket")
            .call(
                "websocket_send_message",
                vec![
                    JsValue::U64(callback_id),
                    JsValue::String(message.to_string()),
                ],
            )
            .exec();
    }

    pub fn dom_bulk_update(&self, value: JsJson) {
        self.dom_access()
            .api()
            .get("dom")
            .call("dom_bulk_update", vec![JsValue::Json(value)])
            .exec();
    }

    pub fn dom_access(&self) -> DomAccess {
        DomAccess::new(
            self.panic_message,
            self.arguments.clone(),
            self.fn_dom_access,
        )
    }

    pub fn get_random(&self, min: u32, max: u32) -> u32 {
        let result = self
            .dom_access()
            .api()
            .call("getRandom", vec![JsValue::U32(min), JsValue::U32(max)])
            .fetch();

        if let JsValue::I32(result) = result {
            result as u32
        } else {
            self.panic_message
                .show(format!("api.get_random -> incorrect result {result:?}"));
            min
        }
    }

    pub fn is_browser(&self) -> bool {
        let result = self
            .dom_access()
            .api()
            .call("isBrowser", Vec::new())
            .fetch();

        if let JsValue::True = result {
            return true;
        }

        if let JsValue::False = result {
            return false;
        }

        log::error!("logical value expected");
        false
    }

    pub fn get_env(&self, name: String) -> Option<String> {
        let result = self
            .dom_access()
            .api()
            .call("get_env", vec![JsValue::String(name)])
            .fetch();

        if let JsValue::Null = result {
            return None;
        }

        if let JsValue::String(value) = result {
            return Some(value);
        }

        log::error!("get_env: string or null was expected");
        None
    }

    /// Synthetic command to respond with plain text, not DOM
    pub fn plain_response(&self, body: String) {
        if self.is_browser() {
            return;
        }

        self.dom_access()
            .synthetic("plain_response", JsValue::String(body))
            .exec();
    }

    /// Synthetic command to respond with custom status code from SSR
    pub fn set_status(&self, status: u16) {
        if self.is_browser() {
            return;
        }

        self.dom_access()
            .synthetic("set_status", JsValue::U32(status as u32))
            .exec();
    }
}
