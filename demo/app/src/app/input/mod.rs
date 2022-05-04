use vertigo::{css_fn, html, VDomElement, Value, VDomComponent, bind};

#[derive(Clone)]
pub struct State {
    pub value: Value<String>,
}

impl State {
    pub fn component() -> VDomComponent {
        let state = State {
            value: Value::new(String::from("")),
        };

        VDomComponent::from(state, render)
    }

    // pub fn increment(&self) {
    //     self.counter.setValue(*self.counter.getValue() + 1);
    // }

    // pub fn decrement(&self) {
    //     self.counter.setValue(*self.counter.getValue() - 1);
    // }
}

css_fn! { wrapper, "
    border: 1px solid black;
    margin: 20px 0;
    padding: 10px;
" }

css_fn! { input_css, "
    margin-left: 10px;
" }

css_fn! { button_css, "
    margin: 0 10px;
    cursor: pointer;
" }

css_fn! { text_css, "
    width: 600px;
    height: 300px;
    border: 1px solid black;
    padding: 5px;
    margin: 10px;
" }

fn render(state: &State) -> VDomElement {
    let on_set1 = bind(state).call(|state| {
        state.value.set("value 1".into());
    });

    let on_set2 = bind(state).call(|state| {
        state.value.set("value 2".into());
    });

    let on_set3 = bind(state).call_param(|state, new_value: String| {
        state.value.set(new_value);
    });

    let on_set4 = bind(state).call_param(|state, new_value: String| {
        state.value.set(new_value);
    });

    let mouse_in = || {
        log::info!("enter");
    };

    let mouse_out = || {
        log::info!("out");
    };

    let value = state.value.get();

    let count = value.len();

    html! {
        <div css={wrapper()} on_mouse_enter={mouse_in} on_mouse_leave={mouse_out}>
            { "To jest input" }
            <input css={input_css()} value={value.as_str()} on_input={on_set3} />
            <button css={button_css()} on_click={on_set1}>"set 1"</button>
            <button css={button_css()} on_click={on_set2}>"set 2"</button>
            <textarea css={text_css()} on_input={on_set4} value={value.as_str()} />
            <div>"count = " { count }</div>
        </div>
    }
}
