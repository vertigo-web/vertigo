use vertigo::{bind, component, css, dom, Value};

#[component]
pub fn MyInput(value: Value<String>) {
    let mouse_in = || {
        log::info!("enter");
    };

    let mouse_out = || {
        log::info!("out");
    };

    let count = value.map(|inner| inner.len().to_string());

    let wrapper = css!(
        "
        border: 1px solid black;
        margin: 20px 0;
        padding: 10px;
    "
    );

    let fragment_components = dom! {
        <ButtonSet value={&value} value_to_set={"set 1"} />
        <ButtonSet value={&value} value_to_set={"set 2"} />
    };

    dom! {
        <div css={wrapper} on_mouse_enter={mouse_in} on_mouse_leave={mouse_out}>
            "This is input"
            <Input value={&value} />
            { fragment_components }
            <TextArea value={value} />
            <div>
                "count = " {count}
            </div>
        </div>
    }
}

#[component]
fn TextArea(value: Value<String>) {
    let on_input = bind!(value, |new_value: String| {
        value.set(new_value);
    });

    let css = css!(
        "
        width: 600px;
        height: 300px;
        border: 1px solid black;
        padding: 5px;
        margin: 10px;
    "
    );

    dom! {
        <textarea {css} {on_input} {value} />
    }
}

#[component]
fn Input(value: Value<String>) {
    let css = css!(
        "
        margin-left: 10px;
    "
    );

    let on_input = bind!(value, |new_value: String| {
        value.set(new_value);
    });

    dom! {
        <input {css} value={value} {on_input} />
    }
}

#[component]
fn ButtonSet(value: Value<String>, value_to_set: String) {
    let css = css!(
        "
        margin: 0 10px;
        cursor: pointer;
    "
    );

    let on_click = bind!(value, value_to_set, |_| {
        value.set(value_to_set.clone());
    });

    dom! {
        <button {css} {on_click}>
            {value_to_set}
        </button>
    }
}
