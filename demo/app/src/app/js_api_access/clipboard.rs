use vertigo::{bind, component, dom, js, transaction, window, Value};

#[component]
pub fn Clipboard() {
    let text_to_copy = Value::new("Text to copy".to_string());

    let on_change = bind!(text_to_copy, |new_value| {
        text_to_copy.set(new_value);
    });

    let copy = bind!(text_to_copy, || {
        transaction(|ctx| {
            let text = text_to_copy.get(ctx);
            window!("navigator.clipboard.writeText()", text);

            js!( navigator.clipboard.writeText(text) );
        })
    });

    dom! {
        <div>
            <p>"Text to copy: "</p>
            <input value={text_to_copy} {on_change} />
            <button on_click={copy} />
        </div>
    }
}
