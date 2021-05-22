use crate::html;

// Make crate available by its name for html macro
use crate as vertigo_html;

use super::utils::*;

#[test]
fn div_with_list() {
    let list = vec![
        html! { <input /> },
        html! { <button /> },
    ];

    let div = html! {
        <div>
            "Label "
            { ..list }
        </div>
    };

    assert_eq!(div.name, "div");
    assert_eq!(div.children.len(), 3);

    let inner1 = get_text(&div.children[0]);
    assert_eq!(&inner1.value, "Label ");

    let inner2 = get_node(&div.children[1]);
    assert_empty(&inner2, "input");

    let inner3 = get_node(&div.children[2]);
    assert_empty(&inner3, "button");
}

#[test]
#[ignore]
fn div_with_element_after_list() {
    let list = vec![
        html! { <input /> },
        html! { <button /> },
    ];

    let div = html! {
        <div>
            { ..list }
            "Error"
        </div>
    };

    assert_eq!(div.name, "div");
    assert_eq!(div.children.len(), 3);

    let inner1 = get_node(&div.children[0]);
    assert_empty(&inner1, "input");

    let inner2 = get_node(&div.children[1]);
    assert_empty(&inner2, "button");

    let inner3 = get_text(&div.children[2]);
    assert_eq!(&inner3.value, "Error");
}
