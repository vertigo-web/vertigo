use vertigo::{main, css, bind, DomNode, dom, Value};

mod light;
use light::Light;

#[derive(Clone, PartialEq)]
pub enum LightState {
    Red,
    RedYellow,
    Green,
    Yellow,
}

pub fn app(state: &Value<LightState>) -> DomNode {
    let red_on = state.map(|state| state == LightState::Red || state == LightState::RedYellow);
    let yellow_on = state.map(|state| state == LightState::Yellow || state == LightState::RedYellow);
    let green_on = state.map(|state| state == LightState::Green);

    let next = bind!(state, |_|
        state.change(|value| {
            *value = match value {
                LightState::Red => LightState::RedYellow,
                LightState::RedYellow => LightState::Green,
                LightState::Green => LightState::Yellow,
                LightState::Yellow => LightState::Red,
            };
        })
    );

    let backplate = css!("
        width: 130px;
        background-color: black;
        display: flex;
        flex-direction: column;
    ");

    let button = css!("
        margin: 20px;
    ");


    dom! {
        <html>
            <head />
            <body>
                <div>
                    <div css={backplate}>
                        <Light light_color="#ff0000" dark_color="#440000" on={red_on} />
                        <Light light_color="#ffff00" dark_color="#444400" on={yellow_on} />
                        <Light light_color="#00ff00" dark_color="#004400" on={green_on} />
                    </div>
                    <button css={button} on_click={next}>"Next"</button>
                </div>
            </body>
        </html>
    }
}

#[main]
fn render() -> DomNode {
    let state = Value::new(LightState::Red);
    app(&state)
}
