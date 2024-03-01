use crate::css::next_id::NextId;

use super::super::transform_css;

#[test]
fn test_media() {
    let css = "
    color: red;
    @media screen and (min-width: 400px) {
        color: green;
    };
    ";

    let next_id = NextId::new();

    let (id, selectors) = transform_css(css, &next_id);

    assert_eq!(id, 1);
    assert_eq!(next_id.current(), 1);

    assert_eq!(
        selectors,
        vec!(
            (
                "@media screen and (min-width: 400px)".into(),
                ".autocss_1 { color: green }".into()
            ),
            (".autocss_1".into(), "color: red".into())
        )
    );
}

#[test]
fn test_doubled() {
    let css = "
    width: 40px;

    @media screen and (min-width: 400px) {
        width: 60px;
    };

    @media screen and (min-width: 800px) {
        width: 80px;
    };
    ";

    let next_id = NextId::new();

    let (id, selectors) = transform_css(css, &next_id);

    assert_eq!(id, 1);
    assert_eq!(next_id.current(), 1);

    assert_eq!(
        selectors,
        vec!(
            (
                "@media screen and (min-width: 400px)".into(),
                ".autocss_1 { width: 60px }".into()
            ),
            (
                "@media screen and (min-width: 800px)".into(),
                ".autocss_1 { width: 80px }".into()
            ),
            (".autocss_1".into(), "width: 40px".into())
        )
    );
}

#[test]
fn test_media_and_pseudo_1() {
    let css = "
        @media screen and (min-width: 600px) {
            :hover {
                transform: scale(1.5);
            };
        };
        @media screen and (min-width: 1000px) {
            :hover {
                transform: scale(2);
            };
        };
    ";

    let next_id = NextId::new();

    let (id, selectors) = transform_css(css, &next_id);

    assert_eq!(id, 1);
    assert_eq!(next_id.current(), 1);

    assert_eq!(
        selectors[0],
        (
            "@media screen and (min-width: 600px)".into(),
            ".autocss_1:hover { transform: scale(1.5); }".into()
        )
    );
    assert_eq!(
        selectors[1],
        (
            "@media screen and (min-width: 1000px)".into(),
            ".autocss_1:hover { transform: scale(2); }".into()
        ),
    );
}

#[test]
fn test_media_and_pseudo_2() {
    let css = "
    width: 40px;

    @media screen and (min-width: 400px) {
        width: 60px;

        :hover {
            color: red;
        };

        :focus::first-letter {
            color: crimson;
        };
    };

    @media screen and (min-width: 800px) {
        width: 80px;
    };

    @media screen and (max-width: 900px) {
        :focus {
            color: magenta;
        }
    };
    ";

    let next_id = NextId::new();

    let (id, selectors) = transform_css(css, &next_id);

    assert_eq!(id, 1);
    assert_eq!(next_id.current(), 1);

    assert_eq!(
        selectors[0],
        ("@media screen and (min-width: 400px)".into(), ".autocss_1 { width: 60px }\n.autocss_1:hover { color: red; }\n.autocss_1:focus::first-letter { color: crimson; }".into())
    );
    assert_eq!(
        selectors[1],
        (
            "@media screen and (min-width: 800px)".into(),
            ".autocss_1 { width: 80px }".into()
        ),
    );
    assert_eq!(
        selectors[2],
        (
            "@media screen and (max-width: 900px)".into(),
            ".autocss_1:focus { color: magenta; }".into()
        ),
    );
    assert_eq!(selectors[3], (".autocss_1".into(), "width: 40px".into()));
}
