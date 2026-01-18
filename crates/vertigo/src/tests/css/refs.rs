use crate as vertigo;
use crate::{CssGroup, css};

#[test]
fn test_ref() {
    let css1 = css! {"color: black;"};
    let css2 = css! {"
        width: 40px;
        [css1] {
            color: red;
        };
    "};

    assert_eq!(
        css1.groups,
        vec![CssGroup::CssStatic {
            value: "color: black;"
        }],
    );

    assert_eq!(
        css2.groups,
        vec![CssGroup::CssDynamic {
            value: "width: 40px;\n.autocss_1 { color: red; };".to_string()
        }],
    );
}

#[test]
fn test_pseudo_and_ref() {
    let css1 = css! {"color: black;"};
    let css2 = css! {"
        width: 40px;
        :hover [css1] {
            color: red;
        };
    "};

    assert_eq!(
        css2.groups,
        vec![CssGroup::CssDynamic {
            value: "width: 40px;\n:hover .autocss_1 { color: red; };".to_string()
        }],
    );
}

#[test]
fn test_pseudo_and_two_refs() {
    let css1 = css! {"color: black;"};
    let css2 = css! {"background-color: white;"};
    let css3 = css! {"
        width: 40px;
        :hover [css1] [css2] {
            color: red;
        };
    "};

    assert_eq!(
        css3.groups,
        vec![CssGroup::CssDynamic {
            value: "width: 40px;\n:hover .autocss_1 .autocss_2 { color: red; };".to_string()
        }],
    );
}

#[test]
fn test_ref_and_variable() {
    let color = "red";
    let css1 = css! {"color: black;"};
    let css2 = css! {"
        width: 40px;
        [css1] {
            color: {color};
        };
    "};

    assert_eq!(
        css2.groups,
        vec![CssGroup::CssDynamic {
            value: "width: 40px;\n.autocss_1 { color: red; };".to_string()
        }],
    );
}
