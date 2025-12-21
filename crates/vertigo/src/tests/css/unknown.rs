use crate::{css, Css};

use super::utils::*;

// Make crate available by its name for css macro
use crate as vertigo;

#[test]
fn test_unknown_rule() {
    let value = css! {"
        unknown-rule-one: somevalue;
        unknown-rule-two: \"quotedvalue\";
    "};

    assert_eq!(
        get_s(&value),
        "unknown-rule-one: somevalue;\nunknown-rule-two: \"quotedvalue\";"
    )
}

#[test]
fn test_unknown_rule_expression() {
    fn css_factory(color: &str, back_color: &str) -> Css {
        css! {"
            some-color-rule: { color };
            background-color: { back_color };
        "}
    }

    let value = css_factory("red", "#asdf");

    assert_eq!(
        get_d(&value),
        "some-color-rule: red;\nbackground-color: #asdf;"
    )
}

#[test]
fn animation_rules() {
    let value = css! {"
        animation-fill-mode: forwards;
    "};

    assert_eq!(get_s(&value), "animation-fill-mode: forwards;")
}
