use vertigo::{VDomElement, VDomText};

use crate::html;

// Make crate available by its name for html macro
use crate as vertigo_html;

use super::utils::*;

#[test]
fn empty_pre() {
    let el = html! { <pre></pre> };
    assert_empty(&el, "pre");
}

#[test]
fn pre_with_text() {
    // Note the trailing space after "Some text"
    let dom1 = html! {
        <pre>"
            Some text 
        "</pre>
    };

    let dom2 = VDomElement::build("pre")
        .children(vec![
            // Note the trailing space after "Some text"
            VDomText::new("
            Some text 
        ").into(),
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}

#[test]
fn pre_with_tight_expression() {
    let value = String::from("bar");

    let dom1 = html! {
        <pre>"Foo"{value}</pre>
    };

    let dom2 = VDomElement::build("pre")
        .children(vec![
            VDomText::new("Foo").into(),
            VDomText::new("bar").into(),
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}

#[test]
fn pre_with_multiline_expression() {
    // Note the trailing space after "Foo"
    let value = String::from(r"#
        Foo 
            Bar
    ");

    let dom1 = html! {
        <pre>{&value}</pre>
    };

    let dom2 = VDomElement::build("pre")
        .children(vec![
            // Note the trailing space after "Foo"
            VDomText::new(r"#
        Foo 
            Bar
    ").into(),
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}

#[test]
fn pre_with_spaced_expression() {
    let value = String::from("bar");
    let dom1 = html! {
        <pre>"Foo "{value}</pre>
    };

    let dom2 = VDomElement::build("pre")
        .children(vec![
            VDomText::new("Foo ").into(),
            VDomText::new("bar").into(),
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}
