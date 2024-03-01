use crate::css::next_id::NextId;

use super::super::transform_css;

#[test]
fn test_basic1() {
    let css = "cursor: pointer;";

    let next_id = NextId::new();

    let (id, selectors) = transform_css(css, &next_id);

    assert_eq!(
        selectors,
        vec!((".autocss_1".into(), "cursor: pointer".into()))
    );

    assert_eq!(id, 1);
    assert_eq!(next_id.current(), 1);
}

#[test]
fn test_basic2() {
    let css =
        "border: 1px solid black; padding: 10px; background-color: #e0e0e0;margin-bottom: 10px;";

    let next_id = NextId::new();

    let (id, selectors) = transform_css(css, &next_id);

    assert_eq!(
        selectors,
        vec!((
            ".autocss_1".into(),
            "border: 1px solid black; padding: 10px; background-color: #e0e0e0; margin-bottom: 10px".into()
        ))
    );

    assert_eq!(id, 1);
    assert_eq!(next_id.current(), 1);
}

#[test]
fn test_basic3() {
    let css = "
        border:1px solid black;
        margin: 5px 0;
    ";

    let next_id = NextId::new();

    let (id, selectors) = transform_css(css, &next_id);

    assert_eq!(
        selectors,
        vec!((
            ".autocss_1".into(),
            "border: 1px solid black; margin: 5px 0".into()
        ))
    );

    assert_eq!(id, 1);
    assert_eq!(next_id.current(), 1);
}
