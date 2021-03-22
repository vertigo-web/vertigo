use crate::html;

// Make crate available by its name for html macro
use crate as vertigo_html;

use super::utils::*;

#[test]
fn empty_textarea() {
    let textarea = html!("
        <textarea></textarea>
    ");

    assert_empty(&textarea, "textarea");
}

#[test]
fn textarea_with_expression() {
    let textarea = html!(r#"
        <textarea>{$ format!("Some {}", "Value") $}</textarea>
    "#);

    assert_eq!(textarea.name, "textarea");

    let text = get_text(&textarea.children[0]);
    assert_eq!(text.value, "Some Value");
}

#[test]
fn div_with_textarea() {
    let div = html!("
        <div>
            Label
            <textarea>Some Value</textarea>
        </div>
    ");

    assert_eq!(div.name, "div");
    assert_eq!(div.children.len(), 2);

    let label = get_text(&div.children[0]);
    assert_eq!(label.value, "Label ");

    let textarea = get_node(&div.children[1]);
    assert_eq!(textarea.name, "textarea");

    let text = get_text(&textarea.children[0]);
    assert_eq!(text.value, "Some Value");
}
