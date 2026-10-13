use crate::css;

use super::utils::*;

// Make crate available by its name for css macro
use crate as vertigo;

#[test]
fn test_background_url_plain() {
    let value = css! {"
        background-image: url('foo');
    "};

    assert_eq!(get_s(&value), "background-image: url('foo');")
}

#[test]
fn test_background_url_expression() {
    let url = "bar";
    let value = css! {"
        background-image: url({url});
    "};

    assert_eq!(get_d(&value), "background-image: url('bar');")
}

#[test]
fn test_expression_in_the_middle() {
    let color = "red";
    let value = css! {"
        border: 1px solid {color};
    "};

    assert_eq!(get_d(&value), "border: 1px solid red;")
}

#[test]
fn test_expression_inside_function() {
    let deg = 45;
    let width = 10;
    let value = css! {"
        transform: rotate({deg}deg);
        width: calc({width}px + 2em);
    "};

    assert_eq!(
        get_d(&value),
        "transform: rotate(45deg);\nwidth: calc(10px + 2em);"
    )
}

#[test]
fn test_background_url_with_more_values() {
    let url = "bar";
    let value = css! {"
        background: url({url}) no-repeat center;
    "};

    assert_eq!(get_d(&value), "background: url('bar') no-repeat center;")
}

#[test]
fn test_background_url_plain_with_more_values() {
    let value = css! {"
        background: url('foo') no-repeat center;
    "};

    assert_eq!(get_s(&value), "background: url('foo') no-repeat center;")
}

#[test]
fn test_background_url_data() {
    let value = css! {"
        background-image: url(data:image/gif;base64,R0lGODlhAQABAAAAACw=);
    "};

    assert_eq!(
        get_s(&value),
        "background-image: url(data:image/gif;base64,R0lGODlhAQABAAAAACw=);"
    )
}

#[test]
fn test_braces_in_quotes_with_expression() {
    let color = "red";
    let value = css! {"
        content: '{not an expression}';
        color: {color};
    "};

    assert_eq!(
        get_d(&value),
        "content: '{not an expression}';\ncolor: red;"
    )
}
