use std::rc::Rc;
use crate::{
    WebsocketMessageDriver,
    DropResource, JsValue,
};

use crate::driver_module::api::ApiImport;

#[derive(Clone)]
pub struct DriverWebsocket {
    api: Rc<ApiImport>,
}

impl DriverWebsocket {
    pub fn new(api: &Rc<ApiImport>) -> DriverWebsocket {
        DriverWebsocket {
            api: api.clone(),
        }
    }

    pub fn websocket_start(&self, host: String, callback: Box<dyn Fn(WebsocketMessageDriver)>) -> DropResource {
        let (callback_id, drop_callback) = self.api.callback_store.register_with_id(move |callback_id, data| {
            if let JsValue::True = data {
                let connection = WebsocketMessageDriver::Connection { callback_id };
                callback(connection);
                return JsValue::Undefined;
            }

            if let JsValue::String(message) = data {
                callback(WebsocketMessageDriver::Message(message));
                return JsValue::Undefined;
            }

            if let JsValue::False = data {
                callback(WebsocketMessageDriver::Close);
                return JsValue::Undefined;
            }

            log::error!("websocket - unsupported message type received");
            JsValue::Undefined
        });

        self.api.websocket_register_callback(host.as_str(), callback_id.as_u64());

        DropResource::new({
            let api = self.api.clone();

            move || {
                api.websocket_unregister_callback(callback_id.as_u64());
                drop_callback.off();
            }
        })
    }
}
