#![allow(clippy::new_without_default)]
use vertigo::{start_app, Value, VDomComponent, bind, DomElement, dom};

pub struct State {
    pub count: Value<i32>,
}

impl State {
    pub fn new() -> State {
        State {
            count: Value::new(0),
        }
    }
}

pub fn render(state: State) -> DomElement {
    let increment = bind(&state.count).call(|context, count| {
        count.set(count.get(context) + 1)
    });

    let decrement = bind(&state.count).call(|context, count| {
        count.set(count.get(context) - 1)
    });

    let text_value = state.count.map(|value| value.to_string());

    dom! {
        <div>
            <p>
                "Counter: "
                <text computed={text_value} />
            </p>
            <button on_click={decrement}>"-"</button>
            <button on_click={increment}>"+"</button>
        </div>
    }
}

#[no_mangle]
pub fn start_application() {
    start_app(|| {
        let state = State::new();
        VDomComponent::dom(render(state))
    });
}
