use crate::{JsJson, JsJsonSerialize, dev::CallbackId, driver_module::api::api_browser_command};

/// Websocket message type on which a websocket handler operates.
pub enum WebsocketMessage {
    Message(JsJson),
    Connection(WebsocketConnection),
    Close,
}

/// Represents websocket connection.
#[derive(Clone, PartialEq)]
pub struct WebsocketConnection {
    callback_id: CallbackId,
}

impl WebsocketConnection {
    pub fn new(callback_id: CallbackId) -> WebsocketConnection {
        WebsocketConnection { callback_id }
    }

    pub fn send(&self, message: impl JsJsonSerialize) {
        let message = message.to_json();
        api_browser_command().websocket_send_message(self.callback_id, message);
    }
}
