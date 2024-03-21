use crate::css;

use super::utils::*;

// Make crate available by its name for css macro
use crate as vertigo;

#[test]
fn test_empty() {
    let value = css!("");

    assert_eq!(get_s(&value), "")
}

#[test]
fn test_empty2() {
    let value = css!("/**/");
    assert_eq!(get_s(&value), "")
}

#[test]
fn test_quite_long() {
    let value = css!(
        r#"content:"201C" attr(title) "201D";
        font-family: "Times New Roman", Times, serif;
        /* font-size: 1.2em; */
        text-align:center;
        background:#333;
        color:#fff;
        display:block;
        float:left;
        /* width:7em; */
        /* margin: 0.25em 1em 0.5em 0; */
        /* padding:1em; */"#
    );

    assert_eq!(get_s(&value), "content: \"201C\" attr(title) \"201D\";\nfont-family: \"Times New Roman\", Times, serif;\ntext-align: center;\nbackground: #333;\ncolor: #fff;\ndisplay: block;\nfloat: left;")
}
