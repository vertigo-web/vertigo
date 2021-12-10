use crate::Driver;

#[derive(Debug)]
pub enum WebsocketMessageDriver {
    Message(String),
    Connection {
        callback_id: u64,
    },
    Close,
}

pub enum WebsocketMessage {
    Message(String),
    Connection(WebcocketConnection),
    Close,
}

#[derive(PartialEq)]
pub struct WebcocketConnection {
    callback_id: u64,
    driver: Driver,
}

impl WebcocketConnection {
    pub fn new(callback_id: u64, driver: Driver) -> WebcocketConnection {
        WebcocketConnection {
            callback_id,
            driver
        }
    }

    pub fn send(&self, message: impl Into<String>) {
        let message = message.into();
        self.driver.websocket_send_message(self.callback_id, message);
    }
}

