use std::rc::Rc;
use vertigo::{
    DropResource,
    KeyDownEvent,
    Value,
    WebsocketConnection,
    WebsocketMessage,
    bind,
    get_driver,
    dom,
    DomElement,
    DomCommentCreate, transaction
};

#[derive(Clone)]
pub struct ChatState {
    _ws_connect: Rc<DropResource>,

    connect: Value<Option<WebsocketConnection>>,
    messages: Value<Vec<Rc<String>>>,
    input_text: Value<String>,
}

fn add_message(messages: &Value<Vec<Rc<String>>>, message: String) {
    transaction(|context| {
        let prev_list: Vec<Rc<String>> = messages.get(context);
        let mut new_list: Vec<Rc<String>> = Vec::new();

        for item in prev_list.iter() {
            new_list.push(item.clone());
        }

        new_list.push(Rc::new(message));

        messages.set(new_list);
    });
}

impl ChatState {
    pub fn new() -> ChatState {
        let connect = Value::new(None);
        let messages = Value::new(Vec::new());
        let input_text = Value::new(String::from(""));

        let ws_connect = {
            let connect = connect.clone();
            let messages = messages.clone();

            get_driver().websocket(
                "ws://127.0.0.1:3000/ws",
                Box::new(move |message| match message {
                    WebsocketMessage::Connection(connection) => {
                        connect.set(Some(connection));
                        log::info!("socket demo - connect ...");
                    }
                    WebsocketMessage::Message(message) => {
                        log::info!("socket demo - new message {}", message);

                        add_message(&messages, message);
                    }
                    WebsocketMessage::Close => {
                        connect.set(None);
                        log::info!("socket demo - close ...");
                    }
                }),
            )
        };

        ChatState {
            _ws_connect: Rc::new(ws_connect),
            connect,
            messages,
            input_text,
        }
    }

    pub fn render(&self) -> DomElement {
        render(self)
    }

    fn submit(&self) {
        transaction(|context| {
            let connect = self.connect.get(context);
            if let Some(connect) = connect.as_ref() {
                let text = self.input_text.get(context);
                connect.send(text);
                self.input_text.set(String::from(""));
            } else {
                log::error!("missing connection");
            }
        });
    }
}

fn render_status(state: &ChatState) -> DomCommentCreate {
    state.connect.render_value(
        |is_connect| {
            let message = match is_connect.is_some() {
                true => "Connection active",
                false => "disconnect",
            };
        
            dom! {
                <div>
                    { message }
                </div>
            }
        }
    )
}


pub fn render(state: &ChatState) -> DomElement {
    let input_view = render_input_text(state);
    let status_view = render_status(state);

    let list = state.messages.render_list(
        |item| item.clone(),
        |message| {
            dom! {
                <div>
                    { message.clone() }
                </div>
            }
        }
    );

    dom! {
        <div>
            { status_view }
            { list }
            { input_view }
        </div>
    }
}

pub fn render_input_text(state: &ChatState) -> DomElement {
    let state = state.clone();
    let text_value = state.input_text.to_computed();

    let on_input = bind(&state).call_param(|_, state, new_text: String| {
        state.input_text.set(new_text);
    });

    let submit = bind(&state).call(|_, state| {
        state.submit();
    });

    let on_key_down = bind(&state).call_param(|_, state, key: KeyDownEvent| {
        if key.code == "Enter" {
            state.submit();
            return true;
        }
        false
    });

    dom! {
        <div>
            <hr/>
            <div>
                <input type="text" value={text_value} on_input={on_input} on_key_down={on_key_down}/>
                <button on_click={submit}>"Send"</button>
            </div>
        </div>
    }
}
