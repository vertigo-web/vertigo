use crate::{dev::VDomText, Value, VDomElement, html};

// Make crate available by its name for html macro
use crate as vertigo;

#[test]
fn div_with_simple_expression() {
    let dom1 = html! {
        <div>
            { 5 + 5 }
        </div>
    };

    let dom2 = VDomElement::build("div")
        .children(vec![
            VDomText::new("10").into()
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}

#[test]
fn div_with_value_expression() {
    let x = Value::new(6);
    let y = Value::new(3);

    let dom1 = html! {
        <div>
            { *x.get_value() + *y.get_value() }
        </div>
    };

    let dom2 = VDomElement::build("div")
        .children(vec![
            VDomText::new("9").into()
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}

#[test]
fn div_with_rc_string_expression() {
    let title = std::rc::Rc::new(String::from("The Title"));

    let dom1 = html! {
        <div>
            { title }
        </div>
    };

    let dom2 = VDomElement::build("div")
        .children(vec![
            VDomText::new("The Title").into()
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}
