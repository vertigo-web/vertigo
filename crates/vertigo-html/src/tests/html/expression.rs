use vertigo::computed::{Dependencies, Value};

use crate::html;

// Make crate available by its name for html macro
use crate as vertigo_html;

use super::utils::*;

#[test]
fn div_with_simple_expression() {
    let div = html! {
        <div>
            { 5 + 5 }
        </div>
    };

    assert_eq!(div.name, "div");
    assert_eq!(div.children.len(), 1);

    let inner = get_text(&div.children[0]);
    assert_eq!(inner.value, "10")
}

#[test]
fn div_with_value_expression() {
    let x = Value::new(Dependencies::default(), 6);
    let y = Value::new(Dependencies::default(), 3);
    let div = html! {
        <div>
            { *x.get_value() + *y.get_value() }
        </div>
    };

    assert_eq!(div.name, "div");
    assert_eq!(div.children.len(), 1);

    let inner = get_text(&div.children[0]);
    assert_eq!(inner.value, "9")
}

#[test]
fn div_with_rc_string_expression() {
    let title = std::rc::Rc::new(String::from("The Title"));
    let div = html! {
        <div>
            { title }
        </div>
    };

    assert_eq!(div.name, "div");
    assert_eq!(div.children.len(), 1);

    let inner = get_text(&div.children[0]);
    assert_eq!(inner.value, "The Title")
}
