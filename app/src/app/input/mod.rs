use std::cmp::PartialEq;
use vertigo::{
    computed::{
        Computed,
        Dependencies,
        Value
    },
    VDomElement,
    node_attr,
    Css,
};
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

fn wrapper() -> Css {
    Css::one("
        border: 1px solid black;
        margin: 20px 0;
        padding: 10px;
    ")
}

fn input_css() -> Css {
    Css::one("
        margin-left: 10px;
    ")
}

fn button_css() -> Css {
    Css::one("
        margin: 0 10px;
        cursor: pointer;
    ")
}

fn text_css() -> Css {
    Css::one("
        width: 600px;
        height: 300px;
        border: 1px solid black;
        padding: 5px;
        margin: 10px;
    ")
}

pub fn render(state: &Computed<State>) -> VDomElement {
    use node_attr::{build_node, node, css, text, attr, on_click, on_input, on_mouse_enter, on_mouse_leave};

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

    build_node("div", vec!(
        css(wrapper()),
        text("To jest input"),
        on_mouse_enter(mouse_in),
        on_mouse_leave(mouse_out),
        node("input", vec!(
            css(input_css()),
            attr("value", (*value).as_str()),
            on_input(on_set3),
        )),
        node("button", vec!(
            css(button_css()),
            on_click(on_set1),
            text("set 1")
        )),
        node("button", vec!(
            css(button_css()),
            on_click(on_set2),
            text("set 2")
        )),
        node("textarea", vec!(
            css(text_css()),
            attr("value", (*value).as_str()),
            on_input(on_set4),
        )),
        node("div", vec!(
            text(format!("count = {}", count)),
        )),
    ))
}