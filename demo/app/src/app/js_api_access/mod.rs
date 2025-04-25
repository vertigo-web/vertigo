use vertigo::{bind, component, css, document, dom, window, JsValue, Value};

#[derive(Default, PartialEq)]
pub struct State {
    answer: Value<String>,
}

#[component]
pub fn JsApiAccess() {
    let state = State::default();

    let container_css = css!{"
    "};

    let items = (1..201)
        .map(|i| dom! { <li>"List item" {i}</li> });

    let to_bottom = || {
        let max_y = window!("scrollMaxY");
        vertigo::log::info!("max_y = {max_y:?}");
        window!("scrollTo()", 0, max_y);
    };

    let down_smooth = || {
        let max_y = window!("scrollMaxY");
        vertigo::log::info!("max_y = {max_y:?}");
        window!("scrollTo()",
            vec![
                ("top", 100000.into()),
                ("behavior", "smooth".into()),
            ]
        );
    };

    let ask = bind!(state.answer, || {
        let js_answer = window!("prompt()", "How are you?");
        if let JsValue::String(js_answer) = js_answer {
            answer.set(js_answer)
        }
    });

    dom! {
        <div css={container_css}>
            <p>
                <button on_click={to_bottom}>"scroll to bottom (FF)"</button>
                <button on_click={down_smooth}>"scroll down smoothly"</button>
                <button on_click={|| { window!("alert()", document!("URL")); }}>"URL"</button>
                <button on_click={|| { window!("alert()", document!("referrer")); }}>"Referrer"</button>
            </p>
            <p>
                <button on_click={ask}>"Ask"</button>
                " Answer: " {state.answer}
            </p>
            <ol>{..items}</ol>
            <button on_click={|| { window!("scrollTo()", 0, 0); }}>"to top"</button>
        </div>
    }
}
