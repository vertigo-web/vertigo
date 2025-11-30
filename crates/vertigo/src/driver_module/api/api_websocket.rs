use std::rc::Rc;

use vertigo_macro::store;

use crate::{
    computed::DropResource,
    dev::{command::WebsocketMessageFromBrowser, CallbackId},
    WebsocketConnection, WebsocketMessage,
};

use super::{api_browser_command, CallbackStore};

#[store]
pub fn api_websocket() -> Rc<ApiWebsocket> {
    ApiWebsocket::new()
}

pub struct ApiWebsocket {
    store: CallbackStore<WebsocketMessageFromBrowser, ()>,
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
        callback: F,
    ) -> DropResource {
        let host: String = host.into();

        let (callback_id, drop) = self.store.register_with_id(
            move |callback_id, message: WebsocketMessageFromBrowser| match message {
                WebsocketMessageFromBrowser::Connected => {
                    let connection = WebsocketConnection::new(callback_id);
                    callback(WebsocketMessage::Connection(connection));
                }
                WebsocketMessageFromBrowser::Message { message } => {
                    callback(WebsocketMessage::Message(message));
                }
                WebsocketMessageFromBrowser::Disconnected => {
                    callback(WebsocketMessage::Close);
                }
            },
        );

        api_browser_command().websocket_register_callback(&host, callback_id);

        DropResource::new(move || {
            api_browser_command().websocket_unregister_callback(callback_id);
            drop.off();
        })
    }

    pub fn callback(&self, callback: CallbackId, response: WebsocketMessageFromBrowser) {
        self.store.call(callback, response);
    }
}
