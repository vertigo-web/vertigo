use vertigo::{css_fn, css_fn_push, html, VDomElement, bind};

use super::State;

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

pub fn render(simple_counter: &State) -> VDomElement {
    let value = simple_counter.counter.get();

    let click_up = bind(simple_counter).call(|simple_counter| {
        simple_counter.increment();
    });

    let click_down = bind(simple_counter).call(|simple_counter| {
        simple_counter.decrement();
    });

    html! {
        <div css={css_wrapper()}>
            <div css={css_box()}>"Counter value = " { value }</div>
            <button css={css_button()} on_click={click_up}>"up"</button>
            <button css={css_button()} on_click={click_down}>"down"</button>
        </div>
    }
}
