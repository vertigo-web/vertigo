use crate::css::next_id::NextId;

use super::super::{transform_css, transform_css_animation_value};

#[test]
fn test_transform_css_animation_value() {
    let next_id = NextId::new();

    let css_value = "1.0s infinite ease-in-out {
        0% {
            transform: scale(0);
        }
        100% {
            transform: scale(1.0);
            opacity: 0;
        }
    }";

    let (css_parsed, css_document) = transform_css_animation_value(css_value, &next_id);

    assert_eq!(next_id.current(), 1);
    assert_eq!(css_parsed, "1.0s infinite ease-in-out autocss_1 ");

    assert_eq!(
        css_document,
        Some((
            "@keyframes autocss_1".into(),
            "0% {
            transform: scale(0);
        }
        100% {
            transform: scale(1.0);
            opacity: 0;
        }"
            .into()
        ))
    );
}

#[test]
fn test_animation_with_additional_rule() {
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

    let next_id = NextId::new();

    let (id, selectors) = transform_css(css, &next_id);

    assert_eq!(id, 1);
    assert_eq!(next_id.current(), 2);

    assert_eq!(
        selectors,
        vec!(
            ("@keyframes autocss_2".into(), "0% {\n            -webkit-transform: scale(0);\n            transform: scale(0);\n        }\n        100% {\n            -webkit-transform: scale(1.0);\n            transform: scale(1.0);\n            opacity: 0;\n        }".into()),
            (".autocss_1".into(), "width: 40px; animation: 1.0s infinite ease-in-out autocss_2 ".into())
        )
    );
}
