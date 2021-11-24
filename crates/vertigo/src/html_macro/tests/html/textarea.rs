use crate::{VDomElement, VDomText, html};

use super::utils::*;

// Make crate available by its name for html macro
use crate as vertigo;

#[test]
fn empty_textarea() {
    let textarea = html! {
        <textarea />
    };

    assert_empty(&textarea, "textarea");
}

#[test]
fn textarea_with_expression() {
    let value = "Some Value";
    let dom1 = html! {
        <textarea value={value} />
    };

    let dom2 = VDomElement::build("textarea")
        .attr("value", "Some Value");

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
            <textarea value="Some Value" />
        </div>
    };

    let dom2 = VDomElement::build("div")
        .children(vec![
            VDomText::new("Label ").into(),
            VDomElement::build("textarea")
                .attr("value", "Some Value")
                .into(),
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}

// #[test]
// fn textarea_with_body_generates_error() {
//     let dom1 = html! {
//         <textarea>"Some Value"</textarea>
//     };
// }
