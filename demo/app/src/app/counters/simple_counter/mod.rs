use vertigo::{Value, transaction, Computed};
use vertigo::{bind, css, DomElement, dom};

#[derive(Clone)]
pub struct SimpleCounter {
    /// Description of this counter
    pub label: Computed<String>,
    /// Value to be counted
    pub value: Value<u32>,
}

impl SimpleCounter {
    pub fn mount(self) -> DomElement {
        let state = &self;

        let click_up = bind!(state, || {
            state.increment();
        });

        let click_down = bind!(state, || {
            state.decrement();
        });

        let css_wrapper = css!("
            border: 1px solid black;
            margin: 5px 0;
        ");

        let css_box = || css!("
            margin: 5px;
        ");

        let css_button = || css_box().push_str("
            cursor: pointer;
        ");

        dom! {
            <div css={css_wrapper}>
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
