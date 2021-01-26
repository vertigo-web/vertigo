use vertigo::Css;

use crate::{css_fn, css};

use super::utils::*;

#[test]
fn unknown_rule() {
    css_fn! { unknown, {
        unknown-rule-one: somevalue;
        unknown-rule-two: "quotedvalue";
    } }

    let value = unknown();

    assert_eq!(get_s(&value), "unknown-rule-one: somevalue;\nunknown-rule-two: \"quotedvalue\";")
}

#[test]
fn unknown_rule_expression() {
    fn css_factory(color: &str, back_color: &str) -> Css {
        css! {
            some-color-rule: { color };
            background-color: { back_color };
        }
    }

    let value = css_factory("red", "#asdf");

    assert_eq!(get_d(&value), "some-color-rule: red;\nbackground-color: #asdf;")
}
