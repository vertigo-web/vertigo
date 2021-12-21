use crate::{dev::VDomText, VDomElement, html};

// Make crate available by its name for html macro
use crate as vertigo;

#[test]
fn div_with_list() {
    let list = vec![
        html! { <input /> },
        html! { <button /> },
    ];

    let dom1 = html! {
        <div>
            "Label "
            { ..list }
        </div>
    };

    let dom2 = VDomElement::build("div")
        .children(vec![
            VDomText::new("Label ").into(),
            VDomElement::build("input").into(),
            VDomElement::build("button").into(),
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}

#[test]
#[ignore]
fn div_with_element_after_list() {
    let list = vec![
        html! { <input /> },
        html! { <button /> },
    ];

    let dom1 = html! {
        <div>
            { ..list }
            "Error"
        </div>
    };

    let dom2 = VDomElement::build("div")
        .children(vec![
            VDomElement::build("input").into(),
            VDomElement::build("button").into(),
            VDomText::new("Error").into(),
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}
