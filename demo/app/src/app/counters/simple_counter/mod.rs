use vertigo::{Value, transaction};
use vertigo::{css_fn, css_fn_push, bind, DomElement, dom};

css_fn! { css_box, "
    margin: 5px;
" }

css_fn_push! { css_button, css_box, "
    cursor: pointer;
" }

css_fn! { css_wrapper, "
    border: 1px solid black;
    margin: 5px 0;
" }

#[derive(Clone)]
pub struct SimpleCounter {
    pub label: &'static str,
    pub value: Value<u32>,
}

impl SimpleCounter {
    pub fn render(self) -> DomElement {
        let click_up = bind(&self).call(|_, state| {
            state.increment();
        });

        let click_down = bind(&self).call(|_, state| {
            state.decrement();
        });

        dom! {
            <div css={css_wrapper()}>
                <div css={css_box()}>
                    {self.label} " = " {self.value}
                </div>
                <button css={css_button()} on_click={click_up}>"up"</button>
                <button css={css_button()} on_click={click_down}>"down"</button>
            </div>
        }
    }

    pub fn increment(&self) {
        transaction(|context|{
            self.value.set(self.value.get(context) + 1);
        });
    }

    pub fn decrement(&self) {
        transaction(|context|{
            self.value.set(self.value.get(context) - 1);
        });
    }
}
