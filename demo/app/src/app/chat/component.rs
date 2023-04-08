use vertigo::{KeyDownEvent, bind, dom, DomNode, component};

use super::state::ChatState;

#[component]
pub fn Chat(ws_chat: String) {
    let state = ChatState::new(ws_chat);

    let input_view = render_input_text(&state);
    let status_view = render_status(&state);

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

fn render_status(state: &ChatState) -> DomNode {
    state.connect.render_value(
        |is_connect| {
            let message = match is_connect.is_some() {
                true => "Connection active",
                false => "disconnected",
            };

            dom! {
                <div>
                    { message }
                </div>
            }
        }
    )
}


pub fn render_input_text(state: &ChatState) -> DomNode {
    let state = state.clone();
    let text_value = state.input_text.to_computed();

    let on_input = bind!(state, |new_text: String| {
        state.input_text.set(new_text);
    });

    let submit = bind!(state, || {
        state.submit();
    });

    let on_key_down = bind!(state, |key: KeyDownEvent| {
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
