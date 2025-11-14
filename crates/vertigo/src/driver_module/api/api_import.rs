use std::rc::Rc;

use crate::{
    driver_module::{
        api::{callbacks::api_callbacks, panic_message::api_panic_message},
        js_value::JsValue,
    },
    struct_mut::ValueMut,
    transaction, DropResource, InstantType, JsJson, WebsocketConnection, WebsocketMessage,
};

use super::api_dom_access::DomAccess;

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

#[derive(Clone, Default)]
pub struct ApiImport {}

impl ApiImport {
    fn console_4(&self, kind: ConsoleLogLevel, arg1: &str, arg2: &str, arg3: &str, arg4: &str) {
        DomAccess::default()
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
        let result = DomAccess::default()
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
            let result = DomAccess::default()
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
            DomAccess::default()
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
        DomAccess::default()
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
        let (callback_id, drop_callback) = api_callbacks().register(move |_| {
            callback();
            JsValue::Undefined
        });

        let result = DomAccess::default()
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

        DropResource::new(move || {
            DomAccess::default()
                .api()
                .get("interval")
                .call("interval_clear", vec![JsValue::I32(timer_id)])
                .exec();

            drop_callback.off();
        })
    }

    pub fn timeout_set<F: Fn() + 'static>(&self, duration: u32, callback: F) -> DropResource {
        let (callback_id, drop_callback) = api_callbacks().register(move |_| {
            callback();
            JsValue::Undefined
        });

        let result = DomAccess::default()
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

        DropResource::new(move || {
            DomAccess::default()
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
        let result = DomAccess::default()
            .root("window")
            .get("Date")
            .call("now", vec![])
            .fetch();

        match result {
            JsValue::I64(time) => time,
            JsValue::F64(time) => time as i64,
            _ => {
                api_panic_message().show(format!("api.utc_now -> incorrect result {result:?}"));
                0_i64
            }
        }
    }

    pub fn timezone_offset(&self) -> i32 {
        let result = DomAccess::default()
            .api()
            .call("getTimezoneOffset", vec![])
            .fetch();

        if let JsValue::I32(result) = result {
            // Return in seconds to be compatible with chrono
            // Opposite as JS returns the offset backwards
            result * -60
        } else {
            api_panic_message().show(format!(
                "api.timezone_offset -> incorrect result {result:?}"
            ));
            0
        }
    }

    pub fn history_back(&self) {
        DomAccess::default()
            .root("window")
            .get("history")
            .call("back", Vec::new())
            .exec();
    }

    ///////////////////////////////////////////////////////////////////////////////////
    // hash router
    ///////////////////////////////////////////////////////////////////////////////////

    pub fn get_hash_location(&self) -> String {
        let result = DomAccess::default()
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
        DomAccess::default()
            .api()
            .get("hashRouter")
            .call("push", vec![JsValue::str(new_hash)])
            .exec();
    }

    pub fn on_hash_change<F: Fn(String) + 'static>(&self, callback: F) -> DropResource {
        let (callback_id, drop_callback) = api_callbacks().register(move |data| {
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

        DomAccess::default()
            .api()
            .get("hashRouter")
            .call("add", vec![JsValue::U64(callback_id.as_u64())])
            .exec();

        DropResource::new(move || {
            DomAccess::default()
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
        let result = DomAccess::default()
            .api()
            .get("historyLocation")
            .call("get", Vec::new())
            .fetch();

        if let JsValue::String(value) = result {
            self.route_from_public(value)
        } else {
            log::error!("historyLocation -> params decode error -> result={result:?}");
            String::from("")
        }
    }

    pub fn push_history_location(&self, new_path: &str) {
        DomAccess::default()
            .api()
            .get("historyLocation")
            .call("push", vec![JsValue::str(new_path)])
            .exec();
    }

    pub fn replace_history_location(&self, new_hash: &str) {
        DomAccess::default()
            .api()
            .get("historyLocation")
            .call("replace", vec![JsValue::str(new_hash)])
            .exec();
    }

    pub fn on_history_change<F: Fn(String) + 'static>(&self, callback: F) -> DropResource {
        let myself = self.clone();
        let (callback_id, drop_callback) = api_callbacks().register(move |data| {
            let new_local_path = if let JsValue::String(new_path) = data {
                myself.route_from_public(new_path.clone())
            } else {
                log::error!("on_history_change -> string was expected -> {data:?}");
                String::from("")
            };

            transaction(|_| {
                callback(new_local_path);
            });

            JsValue::Undefined
        });

        DomAccess::default()
            .api()
            .get("historyLocation")
            .call("add", vec![JsValue::U64(callback_id.as_u64())])
            .exec();

        DropResource::new(move || {
            DomAccess::default()
                .api()
                .get("historyLocation")
                .call("remove", vec![JsValue::U64(callback_id.as_u64())])
                .exec();

            drop_callback.off();
        })
    }

    ///////////////////////////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////////////////////////////////

    #[must_use]
    pub fn websocket<F: Fn(WebsocketMessage) + 'static>(
        &self,
        host: impl Into<String>,
        callback: F,
    ) -> DropResource {
        let host: String = host.into();

        let api = self.clone();

        let (callback_id, drop_callback) =
            api_callbacks().register_with_id(move |callback_id, data| {
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
        DomAccess::default()
            .api()
            .get("websocket")
            .call(
                "websocket_register_callback",
                vec![JsValue::String(host.to_string()), JsValue::U64(callback_id)],
            )
            .exec();
    }

    fn websocket_unregister_callback(&self, callback_id: u64) {
        DomAccess::default()
            .api()
            .get("websocket")
            .call(
                "websocket_unregister_callback",
                vec![JsValue::U64(callback_id)],
            )
            .exec();
    }

    pub fn websocket_send_message(&self, callback_id: u64, message: &str) {
        DomAccess::default()
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
        DomAccess::default()
            .api()
            .get("dom")
            .call("dom_bulk_update", vec![JsValue::Json(value)])
            .exec();
    }

    pub fn get_random(&self, min: u32, max: u32) -> u32 {
        let result = DomAccess::default()
            .api()
            .call("getRandom", vec![JsValue::U32(min), JsValue::U32(max)])
            .fetch();

        if let JsValue::I32(result) = result {
            result as u32
        } else {
            api_panic_message().show(format!("api.get_random -> incorrect result {result:?}"));
            min
        }
    }

    pub fn is_browser(&self) -> bool {
        let result = DomAccess::default()
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
        let result = DomAccess::default()
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

    pub fn route_from_public(&self, path: impl Into<String>) -> String {
        let path: String = path.into();
        if self.is_browser() {
            // In the browser use env variable attached during SSR
            let mount_point = self
                .get_env("vertigo-mount-point".to_string())
                .unwrap_or_else(|| "/".to_string());
            if mount_point != "/" {
                path.trim_start_matches(&mount_point).to_string()
            } else {
                path
            }
        } else {
            // On the server no need to do anything
            path
        }
    }

    /// Synthetic command to respond with custom status code from SSR
    pub fn set_status(&self, status: u16) {
        if self.is_browser() {
            return;
        }

        DomAccess::default()
            .synthetic("set_status", JsValue::U32(status as u32))
            .exec();
    }
}

use vertigo_macro::store;

#[store]
pub fn api_import() -> ApiImport {
    ApiImport::default()
}
