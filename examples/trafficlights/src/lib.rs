use vertigo::{start_app, bind, css_fn, DomElement, dom, Value};

mod light;
use light::Light;

#[derive(Clone, PartialEq)]
enum LightState {
    Red,
    RedYellow,
    Green,
    Yellow,
}

pub fn app() -> DomElement {
    let state = Value::new(LightState::Red);
    let red_on = state.map(|state| state == LightState::Red || state == LightState::RedYellow);
    let yellow_on = state.map(|state| state == LightState::Yellow || state == LightState::RedYellow);
    let green_on = state.map(|state| state == LightState::Green);

    let next = bind!(state, ||
        state.change(|value| {
            *value = match value {
                LightState::Red => LightState::RedYellow,
                LightState::RedYellow => LightState::Green,
                LightState::Green => LightState::Yellow,
                LightState::Yellow => LightState::Red,
            };
        })
    );

    dom! {
        <div>
            <div css={backplate()}>
                <Light light_color="#ff0000" dark_color="#440000" on={red_on} />
                <Light light_color="#ffff00" dark_color="#444400" on={yellow_on} />
                <Light light_color="#00ff00" dark_color="#004400" on={green_on} />
            </div>
            <button css={button()} on_click={next}>"Next"</button>
        </div>
    }
}

css_fn! { backplate, "
    width: 130px;
    background-color: black;
    display: flex;
    flex-direction: column;
" }

css_fn! { button, "
    margin: 20px;
" }

#[no_mangle]
pub fn start_application() {
    start_app(app);
}
