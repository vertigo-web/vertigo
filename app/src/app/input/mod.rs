use std::cmp::PartialEq;
use vertigo::{
    computed::{
        Computed,
        Dependencies,
        Value
    },
    VDomElement,
};
use vertigo_html::{Inline, html_component, css_fn};
// use virtualdom::vdom::StateBox::StateBox;

#[derive(PartialEq)]
pub struct State {
    pub value: Value<String>,
}

impl State {
    pub fn new(root: &Dependencies) -> Computed<State> {
        root.new_computed_from(
            State {
                value: root.new_value("".into())
            }
        )
    }

    // pub fn increment(&self) {
    //     self.counter.setValue(*self.counter.getValue() + 1);
    // }

    // pub fn decrement(&self) {
    //     self.counter.setValue(*self.counter.getValue() - 1);
    // }
}

css_fn! { wrapper, {
    border: 1px solid black;
    margin: 20px 0;
    padding: 10px;
}}

css_fn! { input_css, {
    margin-left: 10px;
}}

css_fn! { button_css, {
    margin: 0 10px;
    cursor: pointer;
}}

css_fn! { text_css, {
    width: 600px;
    height: 300px;
    border: 1px solid black;
    padding: 5px;
    margin: 10px;
}}

pub fn render(state: &Computed<State>) -> VDomElement {
    let state = state.get_value();

    let on_set1 = {
        let value = state.value.clone();

        move || {
            value.set_value("value 1".into());
        }
    };

    let on_set2 = {
        let value = state.value.clone();

        move || {
            value.set_value("value 2".into());
        }
    };

    let on_set3 = {
        let state = state.clone();
        move |new_value: String| {
            let value = state.value.clone();
            value.set_value(new_value);
        }
    };

    let on_set4 = {
        let state = state.clone();
        move |new_value: String| {
            let value = state.value.clone();
            value.set_value(new_value);
        }
    };

    let mouse_in = || {
        log::info!("enter");
    };

    let mouse_out = || {
        log::info!("out");
    };

    let value = state.value.get_value();

    let count = value.len();

    html_component! {
        <div css={wrapper()} onMouseEnter={mouse_in} onMouseLeave={mouse_out}>
            { "To jest input" }
            <input css={input_css()} value={(*value).as_str()} onInput={on_set3} />
            <button css={button_css()} onClick={on_set1}>set 1</button>
            <button css={button_css()} onClick={on_set2}>set 2</button>
            <textarea css={text_css()} onInput={on_set4}>
                { (*value).as_str() }
            </textarea>
            <div>{"count = "} { count }</div>
        </div>
    }
}
