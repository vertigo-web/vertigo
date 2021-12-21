use crate::{dev::VDomText, VDomElement, html};

// Make crate available by its name for html macro
use crate as vertigo;

#[test]
fn empty_div() {
    let dom1 = html! { <div></div> };
    let dom2 = VDomElement::build("div");

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2),
    );
}

#[test]
fn div_with_text() {
    let dom1 = html! {
        <div>
            "Some text"
        </div>
    };

    let dom2 = VDomElement::build("div")
        .children(vec![
            VDomText::new("Some text").into()
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2),
    );
}

#[test]
fn div_with_div() {
    let dom1 = html! {
        <div>
            <div></div>
        </div>
    };

    let dom2 = VDomElement::build("div")
        .children(vec![
            VDomElement::build("div").into()
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2),
    );
}

#[test]
fn div_with_selfclosing_div() {
    let dom1 = html! {
        <div>
            <div />
        </div>
    };

    let dom2 = VDomElement::build("div")
        .children(vec![
            VDomElement::build("div").into()
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2),
    );
}

#[test]
fn div_children_spacing() {
    let value1 = std::rc::Rc::new(String::from("Value 1"));
    let value2 = std::rc::Rc::new(String::from("Value 2"));
    let dom1 = html! {
        <div>
            "Text1 "
            { value1 }
            " Text2 "
            { value2 }
            " Text3"
        </div>
    };

    let dom2 = VDomElement::build("div")
        .children(vec![
            VDomText::new("Text1 ").into(),
            VDomText::new("Value 1").into(),
            VDomText::new(" Text2 ").into(),
            VDomText::new("Value 2").into(),
            VDomText::new(" Text3").into(),
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2),
    );
}

#[test]
fn div_unpacked_children_spacing() {
    let value1 = std::rc::Rc::new(String::from("Value 1"));
    let elems = vec![
        html! { <div>"1"</div> },
        html! { <div>"2"</div> },
    ];
    let dom1 = html! {
        <div>
            "Text1 "
            { value1 }
            " Text2 "
            { ..elems }
        </div>
    };

    let dom2 = VDomElement::build("div")
        .children(vec![
            VDomText::new("Text1 ").into(),
            VDomText::new("Value 1").into(),
            VDomText::new(" Text2 ").into(),
            VDomElement::build("div")
                .children(vec![
                    VDomText::new("1").into(),
                ]).into(),
            VDomElement::build("div")
                .children(vec![
                    VDomText::new("2").into(),
                ]).into(),
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2),
    );
}

#[test]
fn div_no_spacing() {
    let value = std::rc::Rc::new(String::from("Value"));
    let dom1 = html! {
        <div>
            { value }"Text"
        </div>
    };

    let dom2 = VDomElement::build("div")
        .children(vec![
            VDomText::new("Value").into(),
            VDomText::new("Text").into(),
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}
