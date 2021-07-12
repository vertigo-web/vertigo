use super::state::State;

use vertigo::{
    computed::Computed,
    VDomElement,
};

use vertigo_html::{html, css_fn, css_fn_push};

css_fn! { css_box, "
    margin: 5px;
" }

css_fn_push! { css_button, css_box, "
    cursor: pointer;
" }

/*
        &:hover {
            color: red;
        }
*/

css_fn! { css_wrapper, "
    border: 1px solid black;
    margin: 5px 0;
" }

pub fn render(simple_counter: &Computed<State>) -> VDomElement {
    let simple_counter = simple_counter.get_value();
    let value = *(simple_counter.counter.get_value());

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

    html! {
        <div css={css_wrapper()}>
            <div css={css_box()}>"Counter value = " { value }</div>
            <button css={css_button()} on_click={click_up}>"up"</button>
            <button css={css_button()} on_click={click_down}>"down"</button>
        </div>
    }
}
