use std::rc::Rc;
use vertigo::{
    AutoJsJson, DropResource, JsJson, Value, WebsocketConnection, WebsocketMessage, from_json,
    get_driver, transaction,
};

/// Ten sam kształt JSON co `demo/server/src/client_message.rs` (serde externally tagged).
#[derive(AutoJsJson, Clone, Debug, PartialEq, Eq)]
enum ClientMessage {
    Info { message: String },
    UserMessage { message: String },
}

#[derive(Clone)]
pub struct ChatState {
    pub _ws_connect: Rc<DropResource>,

    pub connect: Value<Option<WebsocketConnection>>,
    pub messages: Value<Vec<Rc<String>>>,
    pub input_text: Value<String>,
}

impl ChatState {
    pub fn new(ws_chat: String) -> ChatState {
        let connect = Value::new(None);
        let messages = Value::new(Vec::new());
        let input_text = Value::new(String::from(""));

        let ws_connect = {
            let connect = connect.clone();
            let messages = messages.clone();

            get_driver().websocket(ws_chat, move |message| match message {
                WebsocketMessage::Connection(connection) => {
                    connect.set(Some(connection));
                    log::info!("socket demo - connect ...");
                }
                WebsocketMessage::Message(message) => {
                    Self::add_message(&messages, message);
                }
                WebsocketMessage::Close => {
                    connect.set(None);
                    log::info!("socket demo - close ...");
                }
            })
        };

        ChatState {
            _ws_connect: Rc::new(ws_connect),
            connect,
            messages,
            input_text,
        }
    }

    pub fn submit(&self) {
        transaction(|context| {
            let connect = self.connect.get(context);
            if let Some(connect) = connect.as_ref() {
                let text = self.input_text.get(context);
                connect.send(ClientMessage::UserMessage { message: text });
                self.input_text.set(String::from(""));
            } else {
                log::error!("missing connection");
            }
        });
    }

    fn add_message(messages: &Value<Vec<Rc<String>>>, message: JsJson) {
        let text = match from_json::<ClientMessage>(message.clone()) {
            Ok(parsed) => match parsed {
                ClientMessage::Info { message } | ClientMessage::UserMessage { message } => message,
            },
            Err(_) => match message {
                JsJson::String(s) => s,
                other => format!("{other:?}"),
            },
        };

        transaction(|context| {
            let prev_list: Vec<Rc<String>> = messages.get(context);
            let mut new_list: Vec<Rc<String>> = Vec::new();

            for item in prev_list.iter() {
                new_list.push(item.clone());
            }

            new_list.push(Rc::new(text));

            messages.set(new_list);
        });
    }
}
