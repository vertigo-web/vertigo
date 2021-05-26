use vertigo::{Css, VDomElement, VDomText};

use crate::html;

#[test]
fn div_with_static_css() {
    fn my_css() -> Css { Css::str("color: green") }

    let dom1 = html! {
        <div css={my_css()}>
            "Some text"
        </div>
    };

    let dom2 = VDomElement::build("div")
        .css(my_css())
        .children(vec![
            VDomText::new("Some text").into()
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}

#[test]
fn div_with_dynamic_css() {
    fn my_css() -> Css { Css::string("color: black".to_string()) }

    let dom1 = html! {
        <div css={my_css()} />
    };

    let dom2 = VDomElement::build("div")
        .css(my_css());

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}

#[test]
fn div_with_multiple_css_groups() {
    fn my_css() -> Css { Css::str("color: black") }

    fn my_second_css() -> Css {
        my_css().push_str("color: white")
    }

    // second css attribute overwrites the first one
    let dom1 = html! {
        <div css={my_css()} css={my_second_css()} />
    };

    let dom2 = VDomElement::build("div")
        .css(my_second_css());

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}

#[test]
fn style_basic() {
    let dom1 = html!{
        <style>"html, body { width: 100%; }"</style>
    };

    let dom2 = VDomElement::build("style")
        .children(
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
