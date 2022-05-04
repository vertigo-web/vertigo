use std::rc::Rc;
use vertigo::{
    html, DropResource, KeyDownEvent,
    VDomElement, Value, WebsocketConnection, WebsocketMessage, VDomComponent, bind, get_driver
};

pub struct ChatState {
    _ws_connect: DropResource,

    connect: Value<Option<WebsocketConnection>>,
    messages: Value<Vec<Rc<String>>>,
    input_text: Value<String>,
}

fn add_message(messages: &Value<Vec<Rc<String>>>, message: String) {
    let prev_list: Vec<Rc<String>> = messages.get();
    let mut new_list: Vec<Rc<String>> = Vec::new();

    for item in prev_list.iter() {
        new_list.push(item.clone());
    }

    new_list.push(Rc::new(message));

    messages.set(new_list);
}

impl ChatState {
    pub fn component() -> VDomComponent {
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

        let state = Rc::new(ChatState {
            _ws_connect: ws_connect,
            connect,
            messages,
            input_text,
        });

        render(state)
    }

    fn submit(&self) {
        let connect = self.connect.get();
        if let Some(connect) = connect.as_ref() {
            let text = self.input_text.get();
            connect.send(text);
            self.input_text.set(String::from(""));
        } else {
            log::error!("missing connection");
        }
    }
}

pub fn render(state: Rc<ChatState>) -> VDomComponent {
    let input_view = VDomComponent::from_ref(&state, render_input_text);
    
    VDomComponent::from(state, move |state_value: &Rc<ChatState>| {
            
        let is_connect = state_value.connect.get().is_some();

        let network_info = match is_connect {
            true => "Connection active",
            false => "disconnect",
        };

        let mut list = Vec::new();

        let messages = state_value.messages.get();
        for message in messages.iter() {
            list.push(html! {
                <div>
                    { message.clone() }
                </div>
            });
        }

        html! {
            <div>
                <div>
                    { network_info }
                </div>
                <div>
                    { ..list }
                </div>
                { input_view.clone() }
            </div>
        }
    })
}

pub fn render_input_text(state: &Rc<ChatState>) -> VDomElement {
    let state = state.clone();
    let text_value = state.input_text.get();

    let on_input = bind(&state).call_param(|state, new_text: String| {
        state.input_text.set(new_text);
    });

    let submit = bind(&state).call(|state| {
        state.submit();
    });

    let on_key_down = bind(&state).call_param(|state, key: KeyDownEvent| {
        if key.code == "Enter" {
            state.submit();
            return true;
        }
        false
    });

    html! {
        <div>
            <hr/>
            <div>
                <input type="text" value={text_value} on_input={on_input} on_key_down={on_key_down}/>
                <button on_click={submit}>"Send"</button>
            </div>
        </div>
    }
}
