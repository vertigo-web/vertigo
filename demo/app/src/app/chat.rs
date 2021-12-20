use std::rc::Rc;
use vertigo::{html, Computed, Driver, DropResource, VDomElement, Value, WebsocketConnection, WebsocketMessage};

#[derive(PartialEq)]
pub struct ChatState {
    _ws_connect: DropResource,

    connect: Value<Option<WebsocketConnection>>,
    messages: Value<Vec<Rc<String>>>,
    input_text: Value<String>,
}

fn add_message(messages: &Value<Vec<Rc<String>>>, message: String) {
    let prev_list: Rc<Vec<Rc<String>>> = messages.get_value();
    let mut new_list: Vec<Rc<String>> = Vec::new();

    for item in prev_list.iter() {
        new_list.push(item.clone());
    }

    new_list.push(Rc::new(message));

    messages.set_value(new_list);
}

impl ChatState {
    pub fn new(driver: &Driver) -> Computed<ChatState> {
        let connect = driver.new_value(None);
        let messages = driver.new_value(Vec::new());
        let input_text = driver.new_value(String::from(""));

        let ws_connect = {
            let connect = connect.clone();
            let messages = messages.clone();

            driver.websocket(
                "ws://127.0.0.1:3000/ws",
                Box::new(move |message| match message {
                    WebsocketMessage::Connection(connection) => {
                        connect.set_value(Some(connection));
                        log::info!("socket demo - connect ...");
                    }
                    WebsocketMessage::Message(message) => {
                        log::info!("socket demo - new message {}", message);

                        add_message(&messages, message);
                    }
                    WebsocketMessage::Close => {
                        connect.set_value(None);
                        log::info!("socket demo - close ...");
                    }
                }),
            )
        };

        driver.new_computed_from(ChatState {
            _ws_connect: ws_connect,
            connect,
            messages,
            input_text,
        })
    }
}

pub fn render(state: &Computed<ChatState>) -> VDomElement {
    let state_value = state.get_value();

    let is_connect = state_value.connect.get_value().is_some();

    let network_info = match is_connect {
        true => "Connection active",
        false => "disconnect",
    };

    let mut list = Vec::new();

    let messages = state_value.messages.get_value();
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
            <component {render_input_text} data={state.clone()} />
        </div>
    }
}

pub fn render_input_text(state: &Computed<ChatState>) -> VDomElement {
    let state = state.get_value();
    let text = state.input_text.get_value();
    let text_value = (*text).clone();

    let on_input = {
        let state = state.clone();
        move |new_text: String| {
            state.input_text.set_value(new_text);
        }
    };

    let on_click = {
        let connect = state.connect.clone();
        let text_value = text_value.clone();
        move || {
            let connect = connect.get_value();
            if let Some(connect) = &*connect {
                connect.send(text_value.clone());
                state.input_text.set_value(String::from(""));
            } else {
                log::error!("missing connection");
            }
        }
    };

    html! {
        <div>
            <hr/>
            <input type="text" value={text_value} on_input={on_input} />
            <div on_click={on_click}>"Send"</div>
        </div>
    }
}
