//! A box opened from a button gives the focus back to the button once it's closed, with
//! `get_driver().remember_focus()`. Without it the focus would go nowhere with the box, and Tab
//! would start again from the top of the page. A popup does it too when closed with Escape, and
//! when Tab or a click beside it closes it, in its `on_blur`, the focus stays where they took it.

use vertigo::{Css, KeyDownEvent, Value, bind, component, css, dom, dom_element, get_driver};

#[component]
pub fn FocusBack() {
    // The popup first, for Tab to go from it to the box's button
    dom! {
        <div>
            <FocusPopup />
            <FocusBox />
        </div>
    }
}

#[component]
fn FocusPopup() {
    let open = Value::new(false);
    let show = bind!(open, |_| open.set(true));

    let popup = open.to_computed().render_value_option(move |shown| {
        shown.then(|| {
            let close = bind!(open, || open.set(false));
            let close_on_escape = bind!(open, |key: KeyDownEvent| {
                if key.code == "Escape" {
                    open.set(false);
                    return true;
                }
                false
            });
            let popup = dom_element! {
                <div aria-label="A popup" css={frame()} on_key_down={close_on_escape}>
                    "Closed with Escape, the popup gives the focus back to its button. Tab or a "
                    "click beside it closes it, and the focus stays where it went. "
                    <button on_blur={close} autofocus="autofocus">"In the popup"</button>
                </div>
            };
            // For as long as the popup is on the page
            popup.append_drop_resource(get_driver().remember_focus());
            popup.into()
        })
    });

    dom! {
        <div>
            <button on_click={show}>"Open a popup"</button>
            {popup}
        </div>
    }
}

#[component]
fn FocusBox() {
    let open = Value::new(false);
    let show = bind!(open, |_| open.set(true));

    let the_box = open.to_computed().render_value_option(move |shown| {
        shown.then(|| {
            let close = bind!(open, |_| open.set(false));
            let the_box = dom_element! {
                <div role="dialog" aria-label="A box" css={frame()}>
                    "Closed, the box gives the focus back to its button. "
                    <button on_click={close} autofocus="autofocus">"Close the box"</button>
                </div>
            };
            // For as long as the box is on the page
            the_box.append_drop_resource(get_driver().remember_focus());
            the_box.into()
        })
    });

    dom! {
        <div>
            <button on_click={show}>"Open a box"</button>
            {the_box}
        </div>
    }
}

fn frame() -> Css {
    css! {"
        width: 420px;
        margin: 6px 0;
        padding: 8px;
        border: 1px solid #ccc;
    "}
}
