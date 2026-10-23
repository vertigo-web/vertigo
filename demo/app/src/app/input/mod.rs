use vertigo::{Value, bind, component, css, dom};

#[component]
pub fn MyInput(value: Value<String>) {
    let mouse_in = || {
        log::info!("enter");
    };

    let mouse_out = || {
        log::info!("out");
    };

    // Chars rather than `len()`, which is a byte count: "żółw" is four characters and seven bytes.
    let count = value.map(|inner| inner.chars().count().to_string());

    let wrapper = css! {"
        border: 1px solid black;
        margin: 20px 0;
        padding: 10px;
    "};

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
            <CheckBox checked={Value::new(false)} />
        </div>
    }
}

/// `checked` from a `Value`, with buttons that set it - which reach the box even after it has
/// been clicked, when the browser stops showing what the attribute says. A click on the box goes
/// back to the `Value` through `on_change_checked`.
#[component]
fn CheckBox(checked: Value<bool>) {
    let attr = checked.map(|checked| checked.then(|| "checked".to_string()));
    let state = checked.map(|checked| format!("checked = {checked}"));

    let on_change_checked = bind!(checked, |now: bool| {
        checked.set(now);
    });

    let check = bind!(checked, |_| {
        checked.set(true);
    });

    let uncheck = bind!(checked, |_| {
        checked.set(false);
    });

    dom! {
        <div>
            <input type="checkbox" checked={attr} {on_change_checked} />
            <button on_click={check}>"check"</button>
            <button on_click={uncheck}>"uncheck"</button>
            <div>{state}</div>
        </div>
    }
}

#[component]
fn TextArea(value: Value<String>) {
    let on_input = bind!(value, |new_value: String| {
        value.set(new_value);
    });

    let css = css! {"
        width: 600px;
        height: 300px;
        border: 1px solid black;
        padding: 5px;
        margin: 10px;
    "};

    dom! {
        <textarea {css} {on_input} {value} />
    }
}

#[component]
fn Input(value: Value<String>) {
    let css = css! {"
        margin-left: 10px;
    "};

    let on_input = bind!(value, |new_value: String| {
        value.set(new_value);
    });

    dom! {
        <input {css} value={value} {on_input} />
    }
}

#[component]
fn ButtonSet(value: Value<String>, value_to_set: String) {
    let css = css! {"
        margin: 0 10px;
        cursor: pointer;
    "};

    let on_click = bind!(value, value_to_set, |_| {
        value.set(value_to_set.clone());
    });

    dom! {
        <button {css} {on_click}>
            {value_to_set}
        </button>
    }
}
