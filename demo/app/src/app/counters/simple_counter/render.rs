use vertigo::{css_fn, css_fn_push, bind, DomElement, dom};

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

pub fn render(simple_counter: State) -> DomElement {
    let click_up = bind(&simple_counter).call(|_, simple_counter| {
        simple_counter.increment();
    });

    let click_down = bind(&simple_counter).call(|_, simple_counter| {
        simple_counter.decrement();
    });

    let value = simple_counter.counter.map(|item| item.to_string());

    dom! {
        <div css={css_wrapper()}>
            <div css={css_box()}>
                "Counter value = "
                <text computed={value} />
            </div>
            <button css={css_button()} on_click={click_up}>"up"</button>
            <button css={css_button()} on_click={click_down}>"down"</button>
        </div>
    }
}
