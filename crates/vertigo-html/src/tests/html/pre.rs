use crate::html;

// Make crate available by its name for html macro
use crate as vertigo_html;

use super::utils::*;

#[test]
fn empty_pre() {
    let el = html!("<pre></pre>");
    assert_empty(&el, "pre");
}

#[test]
fn pre_with_text() {
    // Note the trailing space after "Some text"
    let div = html!("
        <pre>
            Some text 
        </pre>
    ");

    assert_eq!(div.name, "pre");
    assert_eq!(div.children.len(), 1);

    let text = get_text(&div.children[0]);

    // Note the trailing space after "Some text"
    assert_eq!(text.value, "
            Some text 
        ");
}

#[test]
fn pre_with_tight_expression() {
    let value = String::from("bar");
    let div = html!("
        <pre>Foo{value}</pre>
    ");

    assert_eq!(div.name, "pre");
    assert_eq!(div.children.len(), 2);

    let inner = get_text(&div.children[0]);
    assert_eq!(inner.value, "Foo");

    let inner = get_text(&div.children[1]);
    assert_eq!(inner.value, "bar");
}

#[test]
fn pre_with_multiline_expression() {
    // Note the trailing space after "Foo"
    let value = String::from(r"#
        Foo 
            Bar
    ");
    let div = html!("
        <pre>{&value}</pre>
    ");

    assert_eq!(div.name, "pre");
    assert_eq!(div.children.len(), 1);

    let inner = get_text(&div.children[0]);
    assert_eq!(inner.value, value);
}

#[test]
fn pre_with_spaced_expression() {
    let value = String::from("bar");
    let div = html!("
        <pre>Foo {value}</pre>
    ");

    assert_eq!(div.name, "pre");
    assert_eq!(div.children.len(), 2);

    let inner = get_text(&div.children[0]);
    assert_eq!(inner.value, "Foo ");

    let inner = get_text(&div.children[1]);
    assert_eq!(inner.value, "bar");
}
