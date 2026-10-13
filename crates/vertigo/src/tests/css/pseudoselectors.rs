use crate::css;

use super::utils::*;

// Make crate available by its name for css macro
use crate as vertigo;

#[test]
fn test_hover() {
    let value = css! {"
        color: red;
        :hover {
            color: white;
            background-color: blue;
        };
        background-color: black;
    "};

    assert_eq!(
        get_s(&value),
        "color: red;\n:hover { color: white;\nbackground-color: blue; };\nbackground-color: black;"
    )
}

#[test]
fn test_hover_with_params() {
    let param_inner = "blue";
    let param_outer = "black";
    let value = css! {"
        color: red;
        :hover {
            color: white;
            background-color: {param_inner};
        };
        background-color: {param_outer};
    "};

    assert_eq!(
        get_d(&value),
        "color: red;\n:hover { color: white;\nbackground-color: blue; };\nbackground-color: black;"
    )
}

#[test]
fn test_last_of_type() {
    let value = css! {"
        color: red;
        :nth-last-of-type(2) {
            background-color: blue;
        };
    "};

    assert_eq!(
        get_s(&value),
        "color: red;\n:nth-last-of-type(2) { background-color: blue; };"
    )
}

#[test]
fn test_doubled() {
    let value = css! {"
        width: 40px;

        :focus {
            color: red;
        };

        :focus::first-letter {
            color: crimson;
        };
    "};

    assert_eq!(
        get_s(&value),
        "width: 40px;\n:focus { color: red; };\n:focus::first-letter { color: crimson; };"
    )
}

#[test]
fn test_pseudo_element() {
    let value = css! {"
        position: relative;

        ::before {
            content: 'X';
        };

        ::after {
            content: 'Y';
        };
    "};

    assert_eq!(
        get_s(&value),
        "position: relative;\n::before { content: 'X'; };\n::after { content: 'Y'; };"
    )
}

#[test]
fn test_chained_pseudo_classes() {
    let value = css! {"
        :first-child:hover {
            color: red;
        };
    "};

    assert_eq!(get_s(&value), ":first-child:hover { color: red; };")
}

#[test]
fn test_pseudo_class_arguments() {
    let value = css! {"
        :not(:last-child) {
            margin-bottom: 4px;
        };

        :not([disabled]) {
            opacity: 1;
        };
    "};

    assert_eq!(
        get_s(&value),
        ":not(:last-child) { margin-bottom: 4px; };\n:not([disabled]) { opacity: 1; };"
    )
}

#[test]
fn test_selector_list() {
    let value = css! {"
        :hover, :focus-visible {
            outline: 2px solid blue;
        };
    "};

    assert_eq!(
        get_s(&value),
        ":hover, :focus-visible { outline: 2px solid blue; };"
    )
}
