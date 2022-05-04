use crate::get_driver;

#[derive(Debug)]
pub enum WebsocketMessageDriver {
    Message(String),
    Connection { callback_id: u32 },
    Close,
}

/// Websocket message type on which a websocket handler operates.
pub enum WebsocketMessage {
    Message(String),
    Connection(WebsocketConnection),
    Close,
}

/// Represents websocket connection.
#[derive(Clone)]
pub struct WebsocketConnection {
    callback_id: u32,
}

impl PartialEq for WebsocketConnection {
    fn eq(&self, other: &Self) -> bool {
        self.callback_id == other.callback_id
    }
}

impl WebsocketConnection {
    pub fn new(callback_id: u32) -> WebsocketConnection {
        WebsocketConnection { callback_id }
    }

    pub fn send(&self, message: impl Into<String>) {
        let message = message.into();
        get_driver().websocket_send_message(self.callback_id, message);
    }
}
