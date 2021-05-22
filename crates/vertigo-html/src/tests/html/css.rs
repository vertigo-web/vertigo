use vertigo::{Css, VDomElement, VDomText};

use crate::html;

use super::utils::*;

#[test]
fn div_with_static_css() {
    fn my_css() -> Css { Css::str("color: green") }

    let div = html! {
        <div css={my_css()}>
            "Some text"
        </div>
    };

    assert_eq!(div.name, "div");
    assert_eq!(div.children.len(), 1);

    let css_groups = div.css.unwrap().groups;
    assert_eq!(css_groups.len(), 1);
    let css_value = get_static_css(&css_groups[0]);
    assert_eq!(css_value, "color: green");

    let text = get_text(&div.children[0]);
    assert_eq!(text.value, "Some text");
}

#[test]
fn div_with_dynamic_css() {
    fn my_css() -> Css { Css::string("color: black".to_string()) }

    let div = html! {
        <div css={my_css()} />
    };

    assert_empty(&div, "div");

    let css_groups = div.css.unwrap().groups;
    assert_eq!(css_groups.len(), 1);
    let css_value = get_dynamic_css(&css_groups[0]);
    assert_eq!(css_value, "color: black");
}

#[test]
fn div_with_multiple_css_groups() {
    fn my_css() -> Css { Css::str("color: black") }

    fn my_second_css() -> Css {
        my_css().push_str("color: white")
    }

    // second css attribute overwrites the first one
    let div = html! {
        <div css={my_css()} css={my_second_css()} />
    };

    assert_empty(&div, "div");

    let css_groups = div.css.unwrap().groups;
    assert_eq!(css_groups.len(), 2);
    let css_value = get_static_css(&css_groups[0]);
    assert_eq!(css_value, "color: black");
    let css_value = get_static_css(&css_groups[1]);
    assert_eq!(css_value, "color: white");
}

#[test]
fn style_basic() {
    let dom1 = html!{
        <style>"html, body { width: 100%; }"</style>
    };


    let dom2 = VDomElement::build("style")
        .child(
            vec!(
                VDomText::new("html, body { width: 100%; }")
                    .into()
            )
        )
    ;

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}
