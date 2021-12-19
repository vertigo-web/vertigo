use crate::Driver;

#[derive(Debug)]
pub enum WebsocketMessageDriver {
    Message(String),
    Connection { callback_id: u64 },
    Close,
}

/// Websocket message type on which a websocket handler operates.
pub enum WebsocketMessage {
    Message(String),
    Connection(WebsocketConnection),
    Close,
}

/// Represents websocket connection.
#[derive(PartialEq)]
pub struct WebsocketConnection {
    callback_id: u64,
    driver: Driver,
}

impl WebsocketConnection {
    pub fn new(callback_id: u64, driver: Driver) -> WebsocketConnection {
        WebsocketConnection { callback_id, driver }
    }

    pub fn send(&self, message: impl Into<String>) {
        let message = message.into();
        self.driver.websocket_send_message(self.callback_id, message);
    }
}
