use super::state::State;

use vertigo::{
    computed::Computed,
    VDomElement,
    Css
};

use vertigo_html::{Inline, html_component};

fn css_box() -> Css {
    Css::one("
        margin: 5px;
    ")
}

fn css_button() -> Css {
    css_box().push("
        cursor: pointer;
    ")
}

/*
        &:hover {
            color: red;
        }
*/

fn css_wrapper() -> Css {
    Css::one("
        border:1px solid black;
        margin: 5px 0;
    ")
}

pub fn render(simple_counter: &Computed<State>) -> VDomElement {
    let simple_counter = simple_counter.get_value();
    let calue = *(simple_counter.counter.get_value());

    let click_up = {
        let simple_counter = simple_counter.clone();
        move || {
            simple_counter.increment();
        }
    };

    let click_down = {
        move || {
            simple_counter.decrement();
        }
    };

    html_component! {
        <div css={css_wrapper()}>
            <div css={css_box()}>{$ format!("Counter value = {}", calue) $}</div>
            <button css={css_button()} onClick={click_up}>up</button>
            <button css={css_button()} onClick={click_down}>down</button>
        </div>
    }
}
