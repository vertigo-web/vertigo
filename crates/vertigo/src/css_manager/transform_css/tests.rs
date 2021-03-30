use crate::css_manager::next_id::NextId;

use super::{css_split_rows, css_split_rows_pair, transform_css, transform_css_animation_value};

#[test]
fn test_css_split_rows1() {
    let css = "cursor: pointer;";

    assert_eq!(
        css_split_rows(css),
        vec!("cursor: pointer")
    );

    let rows_pairs = css_split_rows_pair(css);
    assert_eq!(
        rows_pairs,
        vec!(
            ("cursor", "pointer".into()),
        )
    )
}

#[test]
fn test_css_split_rows2() {
    let css = "border: 1px solid black; padding: 10px; background-color: #e0e0e0; margin-bottom: 10px ; ";

    assert_eq!(
        css_split_rows(css),
        vec!("border: 1px solid black", "padding: 10px", "background-color: #e0e0e0", "margin-bottom: 10px")
    );

    let rows_pairs = css_split_rows_pair(css);
    assert_eq!(
        rows_pairs,
        vec!(
            ("border", "1px solid black".into()),
            ("padding", "10px".into()),
            ("background-color", "#e0e0e0".into()),
            ("margin-bottom", "10px".into())
        )
    )
}

#[test]
fn test_css_split_rows3() {
    let css = "border: 1px solid black; padding: 10px; background-color: #e0e0e0; margin-bottom: 10px   ";

    let rows = css_split_rows(css);
    assert_eq!(
        &rows,
        &vec!("border: 1px solid black", "padding: 10px", "background-color: #e0e0e0", "margin-bottom: 10px")
    );

    let rows_pairs = css_split_rows_pair(css);
    assert_eq!(
        rows_pairs,
        vec!(
            ("border", "1px solid black".into()),
            ("padding", "10px".into()),
            ("background-color", "#e0e0e0".into()),
            ("margin-bottom", "10px".into())
        )
    )
}

#[test]
fn test_basic1() {
    let css = "cursor: pointer;";

    let mut next_id = NextId::new();

    let (id, selectors) = transform_css(css, &mut next_id);

    assert_eq!(selectors, vec!((".autocss_1".into(), "cursor: pointer".into())));

    assert_eq!(id, 1);
    assert_eq!(next_id.current(), 1);
}

#[test]
fn test_basic2() {
    let css = "border: 1px solid black; padding: 10px; background-color: #e0e0e0;margin-bottom: 10px;";

    let mut next_id = NextId::new();

    let (id, selectors) = transform_css(css, &mut next_id);

    assert_eq!(selectors, vec!(
        (".autocss_1".into(), "border: 1px solid black; padding: 10px; background-color: #e0e0e0; margin-bottom: 10px".into())
    ));

    assert_eq!(id, 1);
    assert_eq!(next_id.current(), 1);
}

#[test]
fn test_basic3() {
    let css = "
        border:1px solid black;
        margin: 5px 0;
    ";


    let mut next_id = NextId::new();

    let (id, selectors) = transform_css(css, &mut next_id);

    assert_eq!(selectors, vec!((".autocss_1".into(), "border: 1px solid black; margin: 5px 0".into())));

    assert_eq!(id, 1);
    assert_eq!(next_id.current(), 1);
}

#[test]
fn test_transform_css_animation_value() {

    let mut next_id = NextId::new();

    let css_value = "1.0s infinite ease-in-out {
        0% {
            transform: scale(0);
        }
        100% {
            transform: scale(1.0);
            opacity: 0;
        }
    }";

    let (css_parsed, css_document) = transform_css_animation_value(css_value, &mut next_id);

    assert_eq!(next_id.current(), 1);
    assert_eq!(css_parsed, "1.0s infinite ease-in-out autocss_1 ");

    assert_eq!(css_document, Some(("@keyframes autocss_1".into(), "0% {
            transform: scale(0);
        }
        100% {
            transform: scale(1.0);
            opacity: 0;
        }".into())));
}

#[test]
fn test_animation() {

    let css = "
    width: 40px;
    animation: 1.0s infinite ease-in-out {
        0% {
            -webkit-transform: scale(0);
            transform: scale(0);
        }
        100% {
            -webkit-transform: scale(1.0);
            transform: scale(1.0);
            opacity: 0;
        }
    };
    ";


    let mut next_id = NextId::new();

    let (id, selectors) = transform_css(css, &mut next_id);

    assert_eq!(id, 1);
    assert_eq!(next_id.current(), 2);

    assert_eq!(selectors, vec!(
        ("@keyframes autocss_2".into(), "0% {\n            -webkit-transform: scale(0);\n            transform: scale(0);\n        }\n        100% {\n            -webkit-transform: scale(1.0);\n            transform: scale(1.0);\n            opacity: 0;\n        }".into()),
        (".autocss_1".into(), "width: 40px; animation: 1.0s infinite ease-in-out autocss_2 ".into())
    ));
}

#[test]
fn test_hover() {

    let css = "
    width: 40px;
    :hover {
        color: red;
    };
    ";


    let mut next_id = NextId::new();

    let (id, selectors) = transform_css(css, &mut next_id);

    assert_eq!(id, 1);
    assert_eq!(next_id.current(), 1);

    assert_eq!(selectors, vec!(
        (".autocss_1:hover".into(), "color: red;".into()),
        (".autocss_1".into(), "width: 40px".into())
    ));
}

#[test]
fn test_of_type() {

    let css = "
    width: 40px;
    :nth-of-type(2) {
        color: red;
    };
    ";


    let mut next_id = NextId::new();

    let (id, selectors) = transform_css(css, &mut next_id);

    assert_eq!(id, 1);
    assert_eq!(next_id.current(), 1);

    assert_eq!(selectors, vec!(
        (".autocss_1:nth-of-type(2)".into(), "color: red;".into()),
        (".autocss_1".into(), "width: 40px".into())
    ));
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

    let mut next_id = NextId::new();

    let (id, selectors) = transform_css(css, &mut next_id);

    assert_eq!(id, 1);
    assert_eq!(next_id.current(), 1);

    assert_eq!(selectors, vec!(
        (".autocss_1:focus".into(), "color: red;".into()),
        (".autocss_1:focus::first-letter".into(), "color: crimson;".into()),
        (".autocss_1".into(), "width: 40px".into())
    ));
}
