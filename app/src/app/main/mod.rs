use vertigo::{
    node_attr,
    VDomElement,
    Css,
    computed::{
        Value,
        Computed,
        Dependencies
    },
};
use vertigo_html::html;

mod spinner;

#[derive(PartialEq)]
pub struct MainState {
    pub value: Value<u32>,
}

impl MainState {
    pub fn new(root: &Dependencies) -> Computed<MainState> {
        root.new_computed_from(MainState {
            value: root.new_value(33),
        })
    }

    pub fn increment(&self) {
        let rr = self.value.get_value();
        self.value.set_value(*rr + 1);
    }

    pub fn decrement(&self) {
        let rr = self.value.get_value();
        self.value.set_value(*rr - 1);
    }

}

fn css_footer(show_color: bool) -> Css {
    let base = Css::one("background-color: yellow;");

    if show_color {
        base.push("color: green;")
    } else {
        base.push("color: blue;")
    }
}

fn css_bg() -> Css {
    Css::one("border: 1px solid black; padding: 10px; background-color: #e0e0e0;margin-bottom: 10px;")
}

fn css_button() -> Css {
    css_bg().push("cursor: pointer;")
}

pub fn main_render(state: &Computed<MainState>) -> VDomElement {
    use node_attr::{css, node, text};
    let state = state.get_value();
    let value = state.value.get_value();

    let on_down = {
        let app_state = state.clone();
        move || {
            app_state.decrement();
        }
    };

    let on_up = {
        let app_state = state.clone();
        move || {
            log::info!("on click");
            app_state.increment();
        }
    };

    let show_color = *value % 2 == 0;

    let footer_dom = if *value % 10 == 0 {
        node("div", vec!(
            text(format!("jakis footer {} {} - BEZKLASIE", *value % 2, *value % 3)),
        ))
    } else {
        node("div", vec!(
            css(css_footer(show_color)),
            text(format!("jakis footer {} {}", *value % 2, *value % 3)),
        ))
    };

    html! {
        <div aaa="one" bbb="two">
            Abudabi
            <div css={css_bg}>
                bla bla bla
                <!-- spinner -->
            </div>
            <div css={css_bg} onClick={on_up.clone()}>
                {$ format!("aktualna wartosc = {}", value) $}
            </div>
            <div css={css_bg}>
                {$ "Aktualna wartość: " $} {$ value $}
            </div>
            <div css={css_button} onClick={on_up}>
                up
            </div>
            <div css={css_button} onClick={on_down}>
                down
            </div>
            <!-- footer_dom -->
        </div>
    }

    // build_node("div", vec!(
    //     attr("aaa", "one"),
    //     attr("bbb", "two"),
    //     node("div", vec!(
    //         css(css_bg()),
    //         text("bla bla bla"),
    //         spinner::spinner(),
    //     )),
    //     node("div", vec!(
    //         on_click(on_up.clone()),
    //         text(format!("aktualna wartosc = {}", value)),
    //     )),
    //     html! { <div css={css_button} onClick={on_up}> up </div> },
    //     node("div", vec!(
    //         css(css_button()),
    //         on_click(on_down),
    //         text("down"),
    //     )),
    //     footer_dom,
    // ))
}