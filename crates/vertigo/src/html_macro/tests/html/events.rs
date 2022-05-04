use crate::computed::Value;
use crate::html;

use super::utils::*;

// Make crate available by its name for html macro
use crate as vertigo;

#[test]
fn div_with_mouse_events() {
    let value = Value::new("".to_string());

    let mouse_in = {
        let value = value.clone();
        move || {
            value.set("mouse in".to_string());
        }
    };

    let mouse_out = {
        let value = value.clone();
        move || {
            value.set("mouse out".to_string());
        }
    };

    let div = html! {
        <div on_mouse_enter={mouse_in} on_mouse_leave={mouse_out} />
    };

    assert_empty(&div, "div");

    div.on_mouse_enter.unwrap()();
    assert_eq!(value.get(), "mouse in");

    div.on_mouse_leave.unwrap()();
    assert_eq!(value.get(), "mouse out");
}
