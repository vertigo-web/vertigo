use vertigo::computed::{Dependencies, Value};
use vertigo::{Css, VDomElement, VDomText};

use crate::html;

use super::utils::*;

#[test]
fn button() {
    let dom1 = html! {
        <button>"Label"</button>
    };

    let dom2 = VDomElement::build("button")
        .children(vec![
            VDomText::new("Label").into()
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}

#[test]
fn clickable_button() {
    let value = Value::new(Dependencies::default(), false);

    let on_click = {
        let value = value.clone();
        move || {
            value.set_value(true);
        }
    };

    let button = html! {
        <button on_click={on_click} />
    };

    assert_empty(&button, "button");

    let click = button.on_click.unwrap();
    assert!(!*value.get_value());
    click();
    assert!(*value.get_value());
}

#[test]
fn button_with_css() {
    fn my_css() -> Css { Css::str("background-color: gray") }

    let dom1 = html! {
        <button css={my_css()}>
            "Some text"
        </button>
    };

    let dom2 = VDomElement::build("button")
        .css(my_css())
        .children(vec![
            VDomText::new("Some text").into()
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}
