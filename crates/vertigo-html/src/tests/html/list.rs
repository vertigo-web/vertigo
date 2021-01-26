use crate::{html_component, html_element};

use super::utils::*;

#[test]
fn div_with_list() {
    let list = vec![
        html_element! { <input /> },
        html_element! { <button /> },
    ];

    let div = html_component! {
        <div>
            Label
            { ..list }
        </div>
    };

    assert_eq!(div.name, "div");
    assert_eq!(div.child.len(), 3);

    let inner1 = get_text(&div.child[0]);
    assert_eq!(&inner1.value, "Label");

    let inner2 = get_node(&div.child[1]);
    assert_empty(&inner2, "input");

    let inner3 = get_node(&div.child[2]);
    assert_empty(&inner3, "button");
}

#[test]
#[ignore]
fn div_with_element_after_list() {
    let list = vec![
        html_element! { <input /> },
        html_element! { <button /> },
    ];

    let div = html_component! {
        <div>
        { ..list }
        Error
        </div>
    };

    assert_eq!(div.name, "div");
    assert_eq!(div.child.len(), 3);

    let inner1 = get_node(&div.child[0]);
    assert_empty(&inner1, "input");

    let inner2 = get_node(&div.child[1]);
    assert_empty(&inner2, "button");

    let inner3 = get_text(&div.child[2]);
    assert_eq!(&inner3.value, "Error");
}
