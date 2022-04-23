use vertigo::{html, Driver, VDomElement, Value, VDomComponent, bind};
use vertigo_browserdriver::start_browser_app;


pub struct State {
    pub count: Value<i32>,
}

impl State {
    pub fn component(driver: &Driver) -> VDomComponent {
        let state = State {
            count: driver.new_value(0),
        };

        VDomComponent::from(state, render)
    }
}

pub fn render(state: &State) -> VDomElement {
    let increment = bind(&state.count).call(|count| {
        count.set_value(*count.get_value() + 1)
    });

    let decrement = bind(&state.count).call(|count| {
        count.set_value(*count.get_value() - 1)
    });

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
    start_browser_app(State::component);
}
