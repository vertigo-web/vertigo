use std::{cell::Cell, rc::Rc};

use crate::{JsJson, JsJsonSerialize, dev::CallbackId, driver_module::api::api_browser_command};

/// Websocket message type on which a websocket handler operates.
pub enum WebsocketMessage {
    /// A text frame, parsed as JSON.
    Message(JsJson),
    /// A binary frame.
    Binary(Vec<u8>),
    Connection(WebsocketConnection),
    Close(WebsocketClose),
}

impl WebsocketMessage {
    /// A frame as the browser passes it on: bytes for a binary frame, JSON for a text one.
    pub(crate) fn frame(data: JsJson) -> WebsocketMessage {
        match data {
            JsJson::Vec(data) => WebsocketMessage::Binary(data),
            message => WebsocketMessage::Message(message),
        }
    }
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

    /// Sends `message` as JSON in a text frame.
    pub fn send(&self, message: impl JsJsonSerialize) {
        let message = message.to_json();
        api_browser_command().websocket_send_message(self.callback_id, message);
    }

    /// Sends `data` in a binary frame.
    pub fn send_binary(&self, data: impl Into<Vec<u8>>) {
        // On the websocket channel bytes always mean a binary frame.
        api_browser_command().websocket_send_message(self.callback_id, JsJson::Vec(data.into()));
    }
}

/// How a websocket closed, after a connection or after an attempt to connect that failed.
///
/// The websocket connects again after a while (see [WebsocketOptions]), unless the handler calls
/// [WebsocketClose::stop_reconnecting].
#[derive(Clone, Debug)]
pub struct WebsocketClose {
    /// The close code, f. ex. 1000 for a normal closure, 1001 when the server or the page goes
    /// away, 1006 when the connection broke without a close frame - so also when an attempt to
    /// connect failed - and 4000-4999 for the app's own codes.
    pub code: u16,
    /// The reason the server gave, often empty.
    pub reason: String,
    /// `false` when an attempt to connect failed.
    pub was_connected: bool,
    stop: Rc<Cell<bool>>,
}

impl WebsocketClose {
    pub(crate) fn new(code: u16, reason: String, was_connected: bool) -> WebsocketClose {
        WebsocketClose {
            code,
            reason,
            was_connected,
            stop: Rc::new(Cell::new(false)),
        }
    }

    /// Don't connect again: the websocket stays closed until the app drops it. F. ex. when the
    /// server closed it with a code that means it won't take this client back.
    pub fn stop_reconnecting(&self) {
        self.stop.set(true);
    }

    pub(crate) fn is_stopped(&self) -> bool {
        self.stop.get()
    }
}

/// How a websocket connects and reconnects, for [Driver::websocket_with_options](crate::Driver::websocket_with_options).
///
/// After a connection closes, the websocket connects again after `reconnect_delay`. Each attempt
/// in a row that fails doubles the wait, up to `max_reconnect_delay`. The actual wait is a random
/// half to whole of it, so that clients cut off together don't all come back at once.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebsocketOptions {
    /// Milliseconds an attempt may take to connect before it counts as failed.
    pub connect_timeout: u32,
    /// Milliseconds before the first attempt to reconnect.
    pub reconnect_delay: u32,
    /// The longest wait between attempts, in milliseconds.
    pub max_reconnect_delay: u32,
}

impl Default for WebsocketOptions {
    fn default() -> Self {
        Self {
            connect_timeout: 5_000,
            reconnect_delay: 1_000,
            max_reconnect_delay: 30_000,
        }
    }
}

impl WebsocketOptions {
    /// Bounds of the wait, in milliseconds, before the next attempt once `failures` attempts in
    /// a row ended without a lasting connection (the first being the connection that closed).
    pub(crate) fn reconnect_delay_range(&self, failures: u32) -> (u32, u32) {
        let doublings = 2_u32.saturating_pow(failures.saturating_sub(1));
        let delay = self
            .reconnect_delay
            .saturating_mul(doublings)
            .min(self.max_reconnect_delay);
        (delay / 2, delay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wait_doubles_with_each_failure_up_to_the_limit() {
        let options = WebsocketOptions::default();

        let ranges: Vec<_> = (1..=7)
            .map(|failures| options.reconnect_delay_range(failures))
            .collect();

        assert_eq!(
            ranges,
            [
                (500, 1_000),
                (1_000, 2_000),
                (2_000, 4_000),
                (4_000, 8_000),
                (8_000, 16_000),
                (15_000, 30_000),
                (15_000, 30_000),
            ]
        );
    }

    #[test]
    fn a_long_outage_stays_at_the_limit() {
        let options = WebsocketOptions::default();

        assert_eq!(options.reconnect_delay_range(1_000), (15_000, 30_000));
    }

    #[test]
    fn the_limit_caps_even_the_first_wait() {
        let options = WebsocketOptions {
            reconnect_delay: 10_000,
            max_reconnect_delay: 3_000,
            ..WebsocketOptions::default()
        };

        assert_eq!(options.reconnect_delay_range(1), (1_500, 3_000));
    }

    #[test]
    fn bytes_are_a_binary_frame_and_anything_else_json() {
        let binary = WebsocketMessage::frame(JsJson::Vec(vec![1, 2, 255]));
        let text = WebsocketMessage::frame(JsJson::String("hello".to_string()));
        let list = WebsocketMessage::frame(vec![1_u32, 2].to_json());

        assert!(matches!(binary, WebsocketMessage::Binary(data) if data == [1, 2, 255]));
        assert!(matches!(text, WebsocketMessage::Message(JsJson::String(s)) if s == "hello"));
        // A JSON list of numbers is still JSON, not bytes.
        assert!(matches!(list, WebsocketMessage::Message(JsJson::List(_))));
    }

    #[test]
    fn stopping_reaches_the_driver_through_any_clone() {
        let close = WebsocketClose::new(4004, "no such world".to_string(), true);
        let given_to_the_app = close.clone();

        given_to_the_app.stop_reconnecting();

        assert!(close.is_stopped());
    }
}
