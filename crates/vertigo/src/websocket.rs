use crate::{dev::CallbackId, ApiImport};

/// Websocket message type on which a websocket handler operates.
pub enum WebsocketMessage {
    Message(String),
    Connection(WebsocketConnection),
    Close,
}

/// Represents websocket connection.
#[derive(Clone)]
pub struct WebsocketConnection {
    api: ApiImport,
    callback_id: CallbackId,
}

impl PartialEq for WebsocketConnection {
    fn eq(&self, other: &Self) -> bool {
        self.callback_id == other.callback_id
    }
}

impl WebsocketConnection {
    pub fn new(api: ApiImport, callback_id: CallbackId) -> WebsocketConnection {
        WebsocketConnection { api, callback_id }
    }

    pub fn send(&self, message: impl Into<String>) {
        let message = message.into();

        self.api
            .websocket_send_message(self.callback_id.as_u64(), message.as_str());
    }
}
