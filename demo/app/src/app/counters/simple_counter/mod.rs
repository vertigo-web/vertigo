use vertigo::{Value, transaction, Computed, component};
use vertigo::{bind, css, dom};

#[component]
pub fn SimpleCounter(label: Computed<String>, value: Value<u32>) {
    let click_up = bind!(value, || {
        transaction(|context|{
            value.set(value.get(context) + 1);
        });
    });

    let click_down = bind!(value, || {
        transaction(|context|{
            value.set(value.get(context) - 1);
        });
    });

    let css_wrapper = css!("
        border: 1px solid black;
        margin: 5px 0;
    ");

    let css_box = || css!("
        margin: 5px;
    ");

    let css_button = || css_box().push_str("
        display: block;
        cursor: pointer;
    ");

    let css_wrapper_buttons = css!("
        display: flex;
    ");

    dom! {
        <div css={css_wrapper}>
            <div css={css_box()}>
                {label} " = " {value}
            </div>
            <div css={css_wrapper_buttons}>
                <button css={css_button()} on_click={click_up}>"up"</button>
                <button css={css_button()} on_click={click_down}>"down"</button>
            </div>
        </div>
    }
}
