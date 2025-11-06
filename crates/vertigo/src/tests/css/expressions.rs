use crate::css;

use super::utils::*;

// Make crate available by its name for css macro
use crate as vertigo;

#[test]
fn test_background_url_plain() {
    let value = css!(
        "
        background-image: url('foo');
    "
    );

    assert_eq!(get_s(&value), "background-image: url('foo');")
}

#[test]
fn test_background_url_expression() {
    let url = "bar";
    let value = css!(
        "
        background-image: url({url});
    "
    );

    assert_eq!(get_d(&value), "background-image: url('bar');")
}
