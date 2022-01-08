use vertigo::{html, Computed, Driver, VDomElement, Value};
use vertigo_browserdriver::start_browser_app;

#[derive(PartialEq)]
pub struct State {
    pub count: Value<i32>,
}

impl State {
    pub fn new(driver: &Driver) -> Computed<State> {
        let state = State {
            count: driver.new_value(0),
        };
        driver.new_computed_from(state)
    }
}

pub fn render(app_state: &Computed<State>) -> VDomElement {
    let state = app_state.get_value();

    let increment = {
        let count = state.count.clone();
        move || count.set_value(*count.get_value() + 1)
    };

    let decrement = {
        let count = state.count.clone();
        move || count.set_value(*count.get_value() - 1)
    };

    html! {
        <div>
            <p>"Counter: " { *state.count.get_value() }</p>
            <button on_click={decrement}>"-"</button>
            <button on_click={increment}>"+"</button>
        </div>
    }
}

#[no_mangle]
pub fn start_application() {
    start_browser_app(State::new, render);
}
