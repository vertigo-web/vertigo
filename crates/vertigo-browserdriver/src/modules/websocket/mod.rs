use std::rc::Rc;
use vertigo::{
    dev::WebsocketMessageDriver,
    DropResource,
};

use crate::{utils::callback_manager::CallbackManagerOwner, api::ApiImport};

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

    pub fn register(&self, callback: Box<dyn Fn(WebsocketMessageDriver)>) -> u32 {
        self.stack.set(callback)
    }

    pub fn remove_callback(&self, callback_id: u32) {
        self.stack.remove(callback_id);
    }

    pub fn trigger_callback(&self, callback_id: u32, message: WebsocketMessageDriver) {
        self.stack.trigger(callback_id, message);
    }
}

#[derive(Clone)]
pub struct DriverWebsocket {
    api: Rc<ApiImport>,
    callback: Callback,
}

impl DriverWebsocket {
    pub fn new(api: &Rc<ApiImport>) -> DriverWebsocket {
        DriverWebsocket {
            api: api.clone(),
            callback: Callback::new()
        }
    }

    pub fn websocket_start(&self, host: String, callback: Box<dyn Fn(WebsocketMessageDriver)>) -> DropResource {
        let callback_id = self.callback.register(callback);
        self.api.websocket_register_callback(host.as_str(), callback_id);

        DropResource::new({
            let callback = self.callback.clone();
            let api = self.api.clone();

            move || {
                api.websocket_unregister_callback(callback_id);
                callback.remove_callback(callback_id);
            }
        })
    }

    pub fn websocket_send_message(&self, callback_id: u32, message: String) {
        self.api.websocket_send_message(callback_id, message.as_str());
    }

    pub fn export_websocket_callback_socket(&self, callback_id: u32) {
        self.callback.trigger_callback(callback_id, WebsocketMessageDriver::Connection { callback_id });
    }

    pub fn export_websocket_callback_message(&self, callback_id: u32, message: String) {
        self.callback.trigger_callback(callback_id, WebsocketMessageDriver::Message(message));
    }

    pub fn export_websocket_callback_close(&self, callback_id: u32) {
        self.callback.trigger_callback(callback_id, WebsocketMessageDriver::Close);
    }
}
