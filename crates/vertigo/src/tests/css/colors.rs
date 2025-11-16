use crate::css;

use super::utils::*;

// Make crate available by its name for css macro
use crate as vertigo;

#[test]
fn test_basic_colors() {
    let value = css! {"
        color: red;
        background-color: #667787;
    "};

    assert_eq!(get_s(&value), "color: red;\nbackground-color: #667787;")
}

#[test]
fn test_hex_value_colors() {
    let value = css! {"
        color: #deadbeef;
        background-color: #01020304;
    "};

    assert_eq!(
        get_s(&value),
        "color: #deadbeef;\nbackground-color: #01020304;"
    )
}

#[test]
fn test_rgb_value_colors() {
    let value = css! {"
        color: rgb(1, 128, 256);
        background-color: rgba(0, 96, 192, 255);
    "};

    assert_eq!(
        get_s(&value),
        "color: rgb(1, 128, 256);\nbackground-color: rgba(0, 96, 192, 255);"
    )
}

#[test]
fn test_hsl_value_colors() {
    let value = css! {"
        color: hsl(1, 50%, 25%);
        background-color: hsla(0, 96%, 19%, 2);
    "};

    assert_eq!(
        get_s(&value),
        "color: hsl(1, 50%, 25%);\nbackground-color: hsla(0, 96%, 19%, 2);"
    )
}
