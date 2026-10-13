use crate::css::next_id::NextId;

use super::super::transform_css;

#[test]
fn test_hover() {
    let css = "
    width: 40px;
    :hover {
        color: red;
    };
    ";

    let next_id = NextId::new();

    let (id, selectors) = transform_css(css, &next_id);

    assert_eq!(id, 1);
    assert_eq!(next_id.current(), 1);

    assert_eq!(
        selectors,
        vec!(
            (".autocss_1:hover".into(), "color: red;".into()),
            (".autocss_1".into(), "width: 40px".into())
        )
    );
}

#[test]
fn test_of_type() {
    let css = "
    width: 40px;
    :nth-of-type(2) {
        color: red;
    };
    ";

    let next_id = NextId::new();

    let (id, selectors) = transform_css(css, &next_id);

    assert_eq!(id, 1);
    assert_eq!(next_id.current(), 1);

    assert_eq!(
        selectors,
        vec!(
            (".autocss_1:nth-of-type(2)".into(), "color: red;".into()),
            (".autocss_1".into(), "width: 40px".into())
        )
    );
}

#[test]
fn test_doubled() {
    let css = "
    width: 40px;

    :focus {
        color: red;
    };

    :focus::first-letter {
        color: crimson;
    };
    ";

    let next_id = NextId::new();

    let (id, selectors) = transform_css(css, &next_id);

    assert_eq!(id, 1);
    assert_eq!(next_id.current(), 1);

    assert_eq!(
        selectors,
        vec!(
            (".autocss_1:focus".into(), "color: red;".into()),
            (
                ".autocss_1:focus::first-letter".into(),
                "color: crimson;".into()
            ),
            (".autocss_1".into(), "width: 40px".into())
        )
    );
}

#[test]
fn test_pseudo_element() {
    let css = "
    width: 40px;

    ::before {
        content: 'X';
    };
    ";

    let next_id = NextId::new();

    let (id, selectors) = transform_css(css, &next_id);

    assert_eq!(id, 1);
    assert_eq!(next_id.current(), 1);

    assert_eq!(
        selectors,
        vec!(
            (".autocss_1::before".into(), "content: 'X';".into()),
            (".autocss_1".into(), "width: 40px".into())
        )
    );
}

#[test]
fn test_selector_list() {
    let css = "
    :hover, :focus {
        color: red;
    };

    :is(:hover, :focus) {
        color: blue;
    };
    ";

    let next_id = NextId::new();

    let (_, selectors) = transform_css(css, &next_id);

    assert_eq!(
        selectors,
        vec!(
            (
                ".autocss_1:hover, .autocss_1:focus".into(),
                "color: red;".into()
            ),
            (
                ".autocss_1:is(:hover, :focus)".into(),
                "color: blue;".into()
            ),
        )
    );
}

#[test]
fn test_referenced_css() {
    // References to other Css are already replaced with their class names here
    let css = "
    .autocss_9 {
        color: red;
    };

    .autocss_9:hover {
        color: blue;
    };

    :hover .autocss_9, :focus .autocss_9 {
        color: white;
    };
    ";

    let next_id = NextId::new();

    let (_, selectors) = transform_css(css, &next_id);

    assert_eq!(
        selectors,
        vec!(
            (".autocss_1 .autocss_9".into(), "color: red;".into()),
            (".autocss_1 .autocss_9:hover".into(), "color: blue;".into()),
            (
                ".autocss_1:hover .autocss_9, .autocss_1:focus .autocss_9".into(),
                "color: white;".into()
            ),
        )
    );
}
