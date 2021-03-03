use vertigo::{VDomElement, VDomText, Css, node_attr::css};

use super::{eq_els, EqResult};

#[test]
fn element() {
    let x = VDomElement::new("div", vec![], vec![]);
    let y = VDomElement::new("div", vec![], vec![]);

    assert_eq!(eq_els(&x, &y), EqResult::Equal);
}

#[test]
#[should_panic(expected = "div != p")]
fn element_panic() {
    let x = VDomElement::new("div", vec![], vec![]);
    let y = VDomElement::new("p", vec![], vec![]);

    assert_eq!(eq_els(&x, &y), EqResult::Equal);
}

#[test]
fn children() {
    let x = VDomElement::new(
        "div",
        vec![],
        vec![
            VDomElement::new("h1", vec![], vec![]).into(),
            VDomElement::new("p", vec![], vec![]).into(),
            VDomText::new("foobar").into(),
        ]
    );
    let y = VDomElement::new(
        "div",
        vec![],
        vec![
            VDomElement::new("h1", vec![], vec![]).into(),
            VDomElement::new("p", vec![], vec![]).into(),
            VDomText::new("foobar").into(),
        ]
    );

    assert_eq!(eq_els(&x, &y), EqResult::Equal);
}

#[test]
#[should_panic(expected = "in position 0: h1 != h2")]
fn children_panic_0() {
    let x = VDomElement::new(
        "div",
        vec![],
        vec![
            VDomElement::new("h1", vec![], vec![]).into(),
            VDomElement::new("p", vec![], vec![]).into(),
            VDomText::new("foobar").into(),
        ]
    );
    let y = VDomElement::new(
        "div",
        vec![],
        vec![
            VDomElement::new("h2", vec![], vec![]).into(),
            VDomElement::new("p", vec![], vec![]).into(),
            VDomText::new("foobar").into(),
        ]
    );

    assert_eq!(eq_els(&x, &y), EqResult::Equal);
}

#[test]
#[should_panic(expected = "in position 1: p != span")]
fn children_panic_1() {
    let x = VDomElement::new(
        "div",
        vec![],
        vec![
            VDomElement::new("h1", vec![], vec![]).into(),
            VDomElement::new("p", vec![], vec![]).into(),
            VDomText::new("foobar").into(),
        ]
    );
    let y = VDomElement::new(
        "div",
        vec![],
        vec![
            VDomElement::new("h1", vec![], vec![]).into(),
            VDomElement::new("span", vec![], vec![]).into(),
            VDomText::new("foobar").into(),
        ]
    );

    assert_eq!(eq_els(&x, &y), EqResult::Equal);
}

#[test]
#[should_panic(expected = "in position 2: Text not equal")]
fn children_panic_2() {
    let x = VDomElement::new(
        "div",
        vec![],
        vec![
            VDomElement::new("h1", vec![], vec![]).into(),
            VDomElement::new("p", vec![], vec![]).into(),
            VDomText::new("foobar").into(),
        ]
    );
    let y = VDomElement::new(
        "div",
        vec![],
        vec![
            VDomElement::new("h1", vec![], vec![]).into(),
            VDomElement::new("p", vec![], vec![]).into(),
            VDomText::new("foobar2").into(),
        ]
    );

    assert_eq!(eq_els(&x, &y), EqResult::Equal);
}

#[test]
fn css_in_children() {
    let x = VDomElement::new(
        "div",
        vec![],
        vec![
            VDomElement::new("h1", vec![css(Css::str("color: white"))], vec![]).into(),
            VDomElement::new("p", vec![], vec![]).into(),
        ]
    );
    let y = VDomElement::new(
        "div",
        vec![],
        vec![
            VDomElement::new("h1", vec![css(Css::str("color: white"))], vec![]).into(),
            VDomElement::new("p", vec![], vec![]).into(),
        ]
    );

    assert_eq!(eq_els(&x, &y), EqResult::Equal);
}

#[test]
#[should_panic(expected = "in position 0: Css static groups not equal")]
fn css_in_children_panic() {
    let x = VDomElement::new(
        "div",
        vec![],
        vec![
            VDomElement::new("h1", vec![css(Css::str("color: black"))], vec![]).into(),
            VDomElement::new("p", vec![], vec![]).into(),
        ]
    );
    let y = VDomElement::new(
        "div",
        vec![],
        vec![
            VDomElement::new("h1", vec![css(Css::str("color: white"))], vec![]).into(),
            VDomElement::new("p", vec![], vec![]).into(),
        ]
    );

    assert_eq!(eq_els(&x, &y), EqResult::Equal);
}
