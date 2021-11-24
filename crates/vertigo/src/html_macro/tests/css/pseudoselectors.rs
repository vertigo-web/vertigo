use crate::{css, css_fn};
use super::utils::*;

// Make crate available by its name for css_fn macro
use crate as vertigo;

#[test]
fn hover_css() {
    css_fn! { css_factory, "
        color: red;
        :hover {
            color: white;
            background-color: blue;
        };
        background-color: black;
    " };

    let value = css_factory();

    assert_eq!(get_s(&value), "color: red;\n:hover { color: white;\nbackground-color: blue; };\nbackground-color: black;")
}

#[test]
fn hover_with_params_css() {
    let param_inner = "blue";
    let param_outer = "black";
    let css_factory = || css! { "
        color: red;
        :hover {
            color: white;
            background-color: {param_inner};
        };
        background-color: {param_outer};
    " };

    let value = css_factory();

    assert_eq!(get_d(&value), "color: red;\n:hover { color: white;\nbackground-color: blue; };\nbackground-color: black;")
}

#[test]
fn last_of_type_css() {
    css_fn! { css_factory, "
        color: red;
        :nth-last-of-type(2) {
            background-color: blue;
        };
    " };

    let value = css_factory();

    assert_eq!(get_s(&value), "color: red;\n:nth-last-of-type(2) { background-color: blue; };")
}

#[test]
fn doubled() {
    css_fn! { css_factory, "
        width: 40px;

        :focus {
            color: red;
        };

        :focus::first-letter {
            color: crimson;
        };
    " };

    let value = css_factory();

    assert_eq!(get_s(&value), "width: 40px;\n:focus { color: red; };\n:focus::first-letter { color: crimson; };")
}
