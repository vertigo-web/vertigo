use vertigo::{Css, JsJson, Value, bind, component, css, dom, js};

mod clipboard;
use clipboard::Clipboard;

mod node_ref;
use node_ref::NodeRef;

#[derive(Default, PartialEq)]
pub struct State {
    answer: Value<String>,
}

#[component]
pub fn JsApiAccess() {
    let state = State::default();

    let items = (1..201).map(|i| dom! { <li>"List item" {i}</li> });

    let to_bottom = |_| {
        let max_y = js! { window.scrollMaxY };
        vertigo::log::info!("max_y = {max_y:?}");
        js! { window.scrollTo(0, max_y) };
    };

    let down_smooth = |_| {
        js! {
            window.scrollTo(
                vec![
                    ("top", 100000.into()),
                    ("behavior", "smooth".into()),
                ]
            )
        };
    };

    let ask = bind!(state.answer, |_| {
        let js_answer = js! { window.prompt("How are you?") };
        if let JsJson::String(js_answer) = js_answer {
            answer.set(js_answer)
        }
    });

    dom! {
        <div tw="flex flex-col gap-2 p-2">
            <Clipboard />
            <NodeRef />
            <hr />
            <p tw="flex gap-3">
                <button on_click={to_bottom}>"scroll to bottom (FF)"</button>
                <button on_click={down_smooth}>"scroll down smoothly"</button>
                <button on_click={|_| { js! { window.alert(js! { document.URL }) }; }}>"URL"</button>
                <button on_click={|_| { js! { window.alert(js! { document.referrer }) }; }}>"Referrer"</button>
            </p>
            <p>
                <button on_click={ask}>"Ask"</button>
                " Answer: " {state.answer}
            </p>
            <ol>{..items}</ol>
            <button on_click={|_| { js! { window.scrollTo(0, 0) }; }}>"to top"</button>
        </div>
    }
}

fn inp_css() -> Css {
    css! {"
        border: 1px solid black;
        padding: 3px;
    "}
}
