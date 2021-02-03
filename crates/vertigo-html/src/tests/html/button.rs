use vertigo::computed::{Dependencies, Value};
use vertigo::Css;

use crate::html_component;

use super::utils::*;

#[test]
fn button() {
    let button = html_component!("
        <button>Label</button>
    ");

    assert_eq!(button.name, "button");
    assert_eq!(button.child.len(), 1);

    let label = get_text(&button.child[0]);
    assert_eq!(label.value, "Label");
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

    let button = html_component!("
        <button onClick={on_click} />
    ");

    assert_empty(&button, "button");

    let click = button.on_click.unwrap();
    assert_eq!(*value.get_value(), false);
    click();
    assert_eq!(*value.get_value(), true);
}

#[test]
fn button_with_css() {
    fn my_css() -> Css { Css::one("background-color: gray") }

    let button = html_component!("
        <button css={my_css()}>
            Some text
        </button>
    ");

    assert_eq!(button.name, "button");
    assert_eq!(button.child.len(), 1);

    let css_groups = button.css.unwrap().groups;
    assert_eq!(css_groups.len(), 1);
    let css_value = get_static_css(&css_groups[0]);
    assert_eq!(css_value, "background-color: gray");

    let text = get_text(&button.child[0]);
    assert_eq!(text.value, "Some text");
}
