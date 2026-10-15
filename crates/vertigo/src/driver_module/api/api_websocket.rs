use std::{cell::Cell, rc::Rc};

use vertigo_macro::store;

use crate::{
    DropResource, JsJson, JsJsonSerialize, WebsocketClose, WebsocketConnection, WebsocketMessage,
    WebsocketOptions,
    dev::{CallbackId, command::WebsocketMessageFromBrowser},
};

use super::{CallbackStore, api_browser_command};

#[store]
pub fn api_websocket() -> Rc<ApiWebsocket> {
    ApiWebsocket::new()
}

/// Websockets of the app. The browser runs the socket and its reconnect loop; when a connection
/// or an attempt to connect ends, this decides whether and when the loop tries again.
pub struct ApiWebsocket {
    /// Gives the milliseconds to wait before the next attempt after a close, `None` to stop.
    store: CallbackStore<WebsocketMessageFromBrowser, Option<u32>>,
}

impl ApiWebsocket {
    fn new() -> Rc<ApiWebsocket> {
        Rc::new(ApiWebsocket {
            store: CallbackStore::new(),
        })
    }

    pub fn websocket<F: Fn(WebsocketMessage) + 'static>(
        &self,
        host: impl Into<String>,
        options: WebsocketOptions,
        callback: F,
    ) -> DropResource {
        let host: String = host.into();
        let connect_timeout = options.connect_timeout;
        // Attempts in a row that ended without a lasting connection.
        let failures = Cell::new(0_u32);

        let (callback_id, drop) = self.store.register_with_id(
            move |callback_id, message: WebsocketMessageFromBrowser| match message {
                WebsocketMessageFromBrowser::Connected => {
                    failures.set(0);
                    let connection = WebsocketConnection::new(callback_id);
                    callback(WebsocketMessage::Connection(connection));
                    None
                }
                WebsocketMessageFromBrowser::Message { message } => {
                    callback(WebsocketMessage::frame(message));
                    None
                }
                WebsocketMessageFromBrowser::Disconnected {
                    code,
                    reason,
                    connected,
                } => {
                    let close = WebsocketClose::new(code, reason, connected);
                    callback(WebsocketMessage::Close(close.clone()));

                    if close.is_stopped() {
                        return None;
                    }

                    failures.set(failures.get().saturating_add(1));
                    let (min, max) = options.reconnect_delay_range(failures.get());
                    Some(api_browser_command().get_random(min, max))
                }
            },
        );

        api_browser_command().websocket_register_callback(&host, callback_id, connect_timeout);

        DropResource::new(move || {
            api_browser_command().websocket_unregister_callback(callback_id);
            drop.off();
        })
    }

    /// Passes a websocket event from the browser to the app. After a close, the answer tells the
    /// browser how many milliseconds to wait before connecting again, or `null` not to.
    pub fn callback(&self, callback: CallbackId, response: WebsocketMessageFromBrowser) -> JsJson {
        match self.store.call(callback, response) {
            Some(delay) => delay.to_json(),
            None => JsJson::Null,
        }
    }
}
