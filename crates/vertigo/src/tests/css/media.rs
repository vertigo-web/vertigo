use crate::css;

use super::utils::*;

// Make crate available by its name for css macro
use crate as vertigo;

#[test]
fn test_media() {
    let value = css! {"
        color: red;
        @media screen and (min-width: 600px) {
            color: white;
            background-color: blue;
        };
        background-color: black;
    "};

    assert_eq!(
        get_s(&value),
        "color: red;\n@media screen and (min-width: 600px) { color: white; background-color: blue; };\nbackground-color: black;"
    )
}

#[test]
fn test_media_with_params() {
    let param_inner = "blue";
    let param_outer = "black";
    let value = css! {"
        color: red;
        @media screen and (min-width: 600px) {
            color: white;
            background-color: {param_inner};
        };
        background-color: {param_outer};
    "};

    assert_eq!(
        get_d(&value),
        "color: red;\n@media screen and (min-width: 600px) { color: white; background-color: blue; };\nbackground-color: black;"
    )
}

#[test]
fn test_media_with_pseudo() {
    let value = css! {"
        color: red;
        @media (30em <= width <= 50em) {
            :nth-last-of-type(2) {
                background-color: blue;
            };
        };
    "};

    assert_eq!(
        get_s(&value),
        "color: red;\n@media (30em <= width <= 50em) { :nth-last-of-type(2) { background-color: blue; }; };"
    )
}

#[test]
fn test_doubled() {
    let value = css! {"
        width: 40px;

        @media only screen and (max-width: 1000px) {
            color: red;
        };

        @media print {
            color: black;
        };
    "};

    assert_eq!(
        get_s(&value),
        "width: 40px;\n@media only screen and (max-width: 1000px) { color: red; };\n@media print { color: black; };"
    )
}
