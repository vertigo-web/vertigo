use vertigo::{
    VDomElement,
    Css,
    computed::{
        Value,
        Computed,
        Dependencies
    },
};
use vertigo_html::{Inline, html_component, html_element, css, css_fn, css_fn_push};

mod spinner;

use spinner::spinner;

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
    let color = if show_color { "green" } else { "blue" };
    css!("
        background-color: yellow;
        color: { color };
    ")
}

css_fn! { css_bg, "
    border: 1px solid black;
    padding: 10px;
    background-color: #e0e0e0;
    margin-bottom: 10px;
" }

css_fn_push! { css_button, css_bg, "
    cursor: pointer;
" }

pub fn main_render(state: &Computed<MainState>) -> VDomElement {
    let state = state.get_value();
    let value = state.value.get_value();

    let on_down = {
        let app_state = state.clone();
        move || {
            app_state.decrement();
        }
    };

    let on_up = {
        move || {
            log::info!("on click");
            state.increment();
        }
    };

    let show_color = *value % 2 == 0;

    let footer_dom = if *value % 10 == 0 {
        html_element!(r#"
            <div>
                {$ format!("jakis footer {} {} - BEZKLASIE", *value % 2, *value % 3) $}
            </div>
        "#)
    } else {
        html_element!(r#"
            <div css={css_footer(show_color)}>
                {$ format!("jakis footer {} {}", *value % 2, *value % 3) $}
            </div>
        "#)
    };

    html_component!(r#"
        <div aaa="one" bbb="two">
            Abudabi
            <div css={css_bg()}>
                {$ if *value > 35 { "terefere kuku" } else { "bla bla bla" } $}
                { spinner() }
            </div>
            <div css={css_bg()} onClick={on_up.clone()}>
                {$ format!("aktualna wartosc = {}", value) $}
            </div>
            <div css={css_bg()}>
                { "Aktualna wartość: " } { value }
            </div>
            <div css={css_button()} onClick={on_up}>
                up
            </div>
            <div css={css_button()} onClick={on_down}>
                down
            </div>
            { footer_dom }
        </div>
    "#)
}
