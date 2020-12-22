use virtualdom::{
    computed::{
        Computed,
        Dependencies,
        Value
    },
    VDomNode,
    NodeAttr,
    Css,
};
// use virtualdom::vdom::StateBox::StateBox;

pub struct State {
    pub value: Value<String>,
}

impl State {
    pub fn new(root: &Dependencies) -> Computed<State> {
        root.newComputedFrom(
            State {
                value: root.newValue("".into())
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

pub fn render(state: &Computed<State>) -> VDomNode {
    use NodeAttr::{buildNode, node, css, text, attr, onClick, onInput};

    let state = state.getValue();

    let on_set1 = {
        let value = state.value.clone();

        move || {
            value.setValue("value 1".into());
        }
    };

    let on_set2 = {
        let value = state.value.clone();

        move || {
            value.setValue("value 2".into());
        }
    };

    let on_set3 = {
        let value = state.value.clone();
        move |new_value: String| {
            log::info!(" nowa wartosc3 {}", new_value);
            value.setValue(new_value);
        }
    };

    let on_set4 = {
        let value = state.value.clone();
        move |new_value: String| {
            log::info!(" nowa wartosc4 {}", new_value);
            value.setValue(new_value);
        }
    };

    let value = state.value.getValue();

    buildNode("div", vec!(
        css(wrapper()),
        text("To jest input"),
        node("input", vec!(
            css(input_css()),
            attr("value", (*value).as_str()),
            onInput(on_set3),
        )),
        node("button", vec!(
            css(button_css()),
            onClick(on_set1),
            text("set 1")
        )),
        node("button", vec!(
            css(button_css()),
            onClick(on_set2),
            text("set 2")
        )),
        node("textarea", vec!(
            css(text_css()),
            attr("value", (*value).as_str()),
            onInput(on_set4),
        ))
    ))
}