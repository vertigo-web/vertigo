use vertigo::{VDomElement, VDomText};

use crate::html;

// Make crate available by its name for html macro
use crate as vertigo_html;

use super::utils::*;

#[test]
fn empty_textarea() {
    let textarea = html! {
        <textarea></textarea>
    };

    assert_empty(&textarea, "textarea");
}

#[test]
fn textarea_with_expression() {
    let dom1 = html! {
        <textarea>{$ format!("Some {}", "Value") $}</textarea>
    };

    let dom2 = VDomElement::build("textarea")
        .children(vec![
            VDomText::new("Some Value").into(),
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}

#[test]
fn div_with_textarea() {
    let dom1 = html! {
        <div>
            "Label "
            <textarea>"Some Value"</textarea>
        </div>
    };

    let dom2 = VDomElement::build("div")
        .children(vec![
            VDomText::new("Label ").into(),
            VDomElement::build("textarea")
                .children(vec![
                    VDomText::new("Some Value").into(),
                ])
                .into(),
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}
