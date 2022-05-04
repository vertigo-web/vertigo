#![allow(clippy::new_without_default)]
use vertigo::{start_app, html, VDomElement, Value, VDomComponent, bind};

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

pub fn render(state: &State) -> VDomElement {
    let increment = bind(&state.count).call(|count| {
        count.set(count.get() + 1)
    });

    let decrement = bind(&state.count).call(|count| {
        count.set(count.get() - 1)
    });

    html! {
        <div>
            <p>"Counter: " { state.count.get() }</p>
            <button on_click={decrement}>"-"</button>
            <button on_click={increment}>"+"</button>
        </div>
    }
}

#[no_mangle]
pub fn start_application() {
    let state = State::new();
    let component = VDomComponent::from(state, render);
    start_app(component);
}
