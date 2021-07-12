use vertigo::computed::{Dependencies, Value};

use crate::html;

use super::utils::*;

#[test]
fn div_with_mouse_events() {
    let value = Value::new(Dependencies::default(), "".to_string());

    let mouse_in = {
        let value = value.clone();
        move || {
            value.set_value("mouse in".to_string());
        }
    };

    let mouse_out = {
        let value = value.clone();
        move || {
            value.set_value("mouse out".to_string());
        }
    };

    let div = html! {
        <div on_mouse_enter={mouse_in} on_mouse_leave={mouse_out} />
    };

    assert_empty(&div, "div");

    div.on_mouse_enter.unwrap()();
    assert_eq!(*value.get_value(), "mouse in");

    div.on_mouse_leave.unwrap()();
    assert_eq!(*value.get_value(), "mouse out");
}
