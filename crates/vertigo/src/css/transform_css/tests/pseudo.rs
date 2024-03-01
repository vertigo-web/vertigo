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
