use vertigo::{component, dom, dom_element, js};

use super::inp_css;

#[component]
pub fn NodeRef() {
    let input = dom_element! {
        <input css={inp_css()} value="text" />
    };

    let input_ref = input.get_ref();

    let focus = move |_| {
        js! { #[input_ref] focus() };
    };

    dom! {
        <div tw="flex gap-1">
            <p>"Input with ref: "</p>
            {input}
            <button on_click={focus}>"Focus"</button>
        </div>
    }
}


    // window
    // js! { window };

    // call()
    // js! { call() };

    // coś.method()
    // js! { coś.method() };

    // root().method().property
    // js! { root().method().property };

