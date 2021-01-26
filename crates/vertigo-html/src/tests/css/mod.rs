use crate::css_fn;

mod animation;
mod colors;
mod unknown;
mod utils;

use utils::*;

#[test]
fn empty_css() {
    css_fn! { empty, { } }

    let value = empty();

    assert_eq!(get_s(&value), "")
}

#[test]
fn quite_long_css() {
    css_fn! { empty, {
        content:"201C" attr(title) "201D";
        font-family: "Times New Roman", Times, serif;
        // font-size: 1.2em;
        text-align:center;
        background:#333;
        color:#fff;
        display:block;
        float:left;
        // width:7em;
        // margin: 0.25em 1em 0.5em 0;
        // padding:1em;
    }}

    let value = empty();

    assert_eq!(get_s(&value), "content: \"201C\" attr(title) \"201D\";\nfont-family: \"Times New Roman\", Times, serif;\ntext-align: center;\nbackground: # 333;\ncolor: # fff;\ndisplay: block;\nfloat: left;")
}
