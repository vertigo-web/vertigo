use vertigo::Css;

use crate::html_component;

use super::utils::*;

#[test]
fn div_with_static_css() {
    fn my_css() -> Css { Css::one("color: green") }

    let div = html_component!("
        <div css={my_css()}>
            Some text
        </div>
    ");

    assert_eq!(div.name, "div");
    assert_eq!(div.child.len(), 1);

    let css_groups = div.css.unwrap().groups;
    assert_eq!(css_groups.len(), 1);
    let css_value = get_static_css(&css_groups[0]);
    assert_eq!(css_value, "color: green");

    let text = get_text(&div.child[0]);
    assert_eq!(text.value, "Some text");
}

#[test]
fn div_with_dynamic_css() {
    fn my_css() -> Css { Css::new("color: black".to_string()) }

    let div = html_component!("
        <div css={my_css()} />
    ");

    assert_empty(&div, "div");

    let css_groups = div.css.unwrap().groups;
    assert_eq!(css_groups.len(), 1);
    let css_value = get_dynamic_css(&css_groups[0]);
    assert_eq!(css_value, "color: black");
}

#[test]
fn div_with_multiple_css_groups() {
    fn my_css() -> Css { Css::one("color: black") }

    fn my_second_css() -> Css {
        my_css().push("color: white")
    }

    // second css attribute overwrites the first one
    let div = html_component!("
        <div css={my_css()} css={my_second_css()} />
    ");

    assert_empty(&div, "div");

    let css_groups = div.css.unwrap().groups;
    assert_eq!(css_groups.len(), 2);
    let css_value = get_static_css(&css_groups[0]);
    assert_eq!(css_value, "color: black");
    let css_value = get_static_css(&css_groups[1]);
    assert_eq!(css_value, "color: white");
}
