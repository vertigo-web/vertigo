use crate::css_fn;

use super::utils::*;

#[test]
fn basic_colors_css() {
    css_fn! { css_factory, {
        color: red;
        background-color: #667787;
    } }

    let value = css_factory();

    assert_eq!(get_s(&value), "color: red;\nbackground-color: #667787;")
}

#[test]
fn hex_value_colors_css() {
    css_fn! { css_factory, {
        color: #deadbeef;
        background-color: #01020304;
    } }

    let value = css_factory();

    assert_eq!(get_s(&value), "color: #deadbeef;\nbackground-color: #01020304;")
}

#[test]
fn rgb_value_colors_css() {
    css_fn! { css_factory, {
        color: rgb(1, 128, 256);
        background-color: rgba(0, 96, 192, 255);
    } }

    let value = css_factory();

    assert_eq!(get_s(&value), "color: rgb(1, 128, 256);\nbackground-color: rgba(0, 96, 192, 255);")
}

#[test]
fn hsl_value_colors_css() {
    css_fn! { css_factory, {
        color: hsl(1, 50%, 25%);
        background-color: hsla(0, 96%, 19%, 2);
    } }

    let value = css_factory();

    assert_eq!(get_s(&value), "color: hsl(1, 50%, 25%);\nbackground-color: hsla(0, 96%, 19%, 2);")
}
