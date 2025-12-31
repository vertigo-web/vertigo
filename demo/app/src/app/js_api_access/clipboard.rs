use vertigo::{Value, bind, component, dom, js, transaction};

use super::inp_css;

#[component]
pub fn Clipboard() {
    let text_to_copy = Value::new("Text to copy".to_string());

    let on_change = bind!(text_to_copy, |new_value| {
        text_to_copy.set(new_value);
    });

    let copy = bind!(text_to_copy, |_| {
        transaction(|ctx| {
            let text = text_to_copy.get(ctx);
            js! { window.navigator.clipboard.writeText(text) };
        })
    });

    let focus = |_| {
        js! { document.getElementById("my_id").focus() };
    };

    dom! {
        <div tw="flex gap-1">
            <p>"Text to copy: "</p>
            <input css={inp_css()} id="my_id" value={text_to_copy} {on_change} />
            <button on_click={copy}>"Copy to clipboard"</button>
            <button on_click={focus}>"Focus"</button>
        </div>
    }
}
