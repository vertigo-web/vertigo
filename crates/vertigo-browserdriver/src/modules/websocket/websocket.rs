use std::rc::Rc;
use vertigo::{
    dev::WebsocketMessageDriver,
    utils::{DropResource},
};
use wasm_bindgen::closure::Closure;

use crate::modules::websocket::js_websocket::DriverWebsocketJs;
use crate::utils::callback_manager::CallbackManagerOwner;

#[derive(Clone)]
struct Callback {
    stack: Rc<CallbackManagerOwner<WebsocketMessageDriver>>,
}

impl Callback {
    pub fn new() -> Callback {
        Callback {
            stack: Rc::new(CallbackManagerOwner::new())
        }
    }

    pub fn register(&self, callback: Box<dyn Fn(WebsocketMessageDriver)>) -> u64 {
        self.stack.set(callback)
    }

    pub fn remove_callback(&self, callback_id: u64) {
        self.stack.remove(callback_id);
    }

    pub fn trigger_callback(&self, callback_id: u64, message: WebsocketMessageDriver) {
        self.stack.trigger(callback_id, message);
    }
}

pub struct DriverWebsocket {
    callback: Callback,
    _callback_socket: Closure<dyn Fn(u64)>,
    _callback_message: Closure<dyn Fn(u64, String)>,
    _callback_close: Closure<dyn Fn(u64)>,
    driver_js: Rc<DriverWebsocketJs>,
}

impl DriverWebsocket {
    pub fn new() -> DriverWebsocket {
        let callback = Callback::new();

        let callback_socket = {
            let callback = callback.clone();
            Closure::new(move |callback_id: u64| {
                callback.trigger_callback(callback_id, WebsocketMessageDriver::Connection { callback_id });
            })
        };

        let callback_message = {
            let callback = callback.clone();
            Closure::new(move |callback_id: u64, message: String| {
                callback.trigger_callback(callback_id, WebsocketMessageDriver::Message(message));
            })
        };

        let callback_close = {
            let callback = callback.clone();
            Closure::new(move |callback_id: u64| {
                callback.trigger_callback(callback_id, WebsocketMessageDriver::Close);
            })
        };

        let driver_js = DriverWebsocketJs::new(&callback_socket, &callback_message, &callback_close);

        DriverWebsocket {
            callback,
            _callback_socket: callback_socket,
            _callback_message: callback_message,
            _callback_close: callback_close,
            driver_js: Rc::new(driver_js),
        }
    }

    pub fn websocket_start(&self, host: String, callback: Box<dyn Fn(WebsocketMessageDriver)>) -> DropResource {
        let callback_id = self.callback.register(callback);
        self.driver_js.register_callback(host, callback_id);

        DropResource::new({
            let driver_js = self.driver_js.clone();
            let callback = self.callback.clone();

            move || {
                driver_js.unregister_callback(callback_id);
                callback.remove_callback(callback_id);
            }
        })
    }

    pub fn websocket_send_message(&self, callback_id: u64, message: String) {
        self.driver_js.send_message(callback_id, message);
    }
}
