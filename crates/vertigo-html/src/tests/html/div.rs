use vertigo::computed::{Dependencies, Value};

use crate::html;

// Make crate available by its name for html macro
use crate as vertigo_html;

use super::utils::*;

#[test]
fn empty_div() {
    let el = html!("<div></div>");
    assert_empty(&el, "div");
}

#[test]
fn div_with_text() {
    let div = html!("
        <div>
            Some text
        </div>
    ");

    assert_eq!(div.name, "div");
    assert_eq!(div.children.len(), 1);

    let text = get_text(&div.children[0]);
    assert_eq!(text.value, "Some text");
}

#[test]
fn div_with_div() {
    let div = html!("
        <div>
            <div></div>
        </div>
    ");

    assert_eq!(div.name, "div");
    assert_eq!(div.children.len(), 1);

    let inner = get_node(&div.children[0]);
    assert_empty(&inner, "div");
}

#[test]
fn div_with_simple_expression() {
    let div = html!("
        <div>
            { 5 + 5 }
        </div>
    ");

    assert_eq!(div.name, "div");
    assert_eq!(div.children.len(), 1);

    let inner = get_text(&div.children[0]);
    assert_eq!(inner.value, "10")
}

#[test]
fn div_with_value_expression() {
    let x = Value::new(Dependencies::default(), 6);
    let y = Value::new(Dependencies::default(), 3);
    let div = html!("
        <div>
            { *x.get_value() + *y.get_value() }
        </div>
    ");

    assert_eq!(div.name, "div");
    assert_eq!(div.children.len(), 1);

    let inner = get_text(&div.children[0]);
    assert_eq!(inner.value, "9")
}

#[test]
fn div_with_rc_string_expression() {
    let title = std::rc::Rc::new(String::from("The Title"));
    let div = html!("
        <div>
            { title }
        </div>
    ");

    assert_eq!(div.name, "div");
    assert_eq!(div.children.len(), 1);

    let inner = get_text(&div.children[0]);
    assert_eq!(inner.value, "The Title")
}

#[test]
fn div_children_spacing() {
    let value1 = std::rc::Rc::new(String::from("Value 1"));
    let value2 = std::rc::Rc::new(String::from("Value 2"));
    let div = html!("
        <div>
            Text1
            { value1 }
            Text2
            { value2 }
            Text3
        </div>
    ");

    assert_eq!(div.name, "div");
    assert_eq!(div.children.len(), 5);

    let inner = get_text(&div.children[0]);
    assert_eq!(inner.value, "Text1 ");
    let inner = get_text(&div.children[1]);
    assert_eq!(inner.value, "Value 1");
    let inner = get_text(&div.children[2]);
    assert_eq!(inner.value, " Text2 ");
    let inner = get_text(&div.children[3]);
    assert_eq!(inner.value, "Value 2");
    let inner = get_text(&div.children[4]);
    assert_eq!(inner.value, " Text3");
}

#[test]
fn div_unpacked_children_spacing() {
    let value1 = std::rc::Rc::new(String::from("Value 1"));
    let elems = vec![
        html!("<div>1</div>"),
        html!("<div>2</div>")
    ];
    let div = html!("
        <div>
            Text1
            { value1 }
            Text2
            { ..elems }
        </div>
    ");

    assert_eq!(div.name, "div");
    assert_eq!(div.children.len(), 5);

    let inner = get_text(&div.children[0]);
    assert_eq!(inner.value, "Text1 ");
    let inner = get_text(&div.children[1]);
    assert_eq!(inner.value, "Value 1");
    let inner = get_text(&div.children[2]);
    assert_eq!(inner.value, " Text2 ");
}

#[test]
fn div_no_spacing() {
    let value = std::rc::Rc::new(String::from("Value"));
    let div = html!("
        <div>
            { value }Text
        </div>
    ");

    assert_eq!(div.name, "div");
    assert_eq!(div.children.len(), 2);

    let inner = get_text(&div.children[0]);
    assert_eq!(inner.value, "Value");
    let inner = get_text(&div.children[1]);
    assert_eq!(inner.value, "Text");
}
