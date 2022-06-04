use crate::{dev::VDomText, Value, VDomElement, html, bind, transaction};

use super::utils::*;

// Make crate available by its name for html macro
use crate as vertigo;

#[test]
fn div_with_label_and_input() {
    let dom1 = html! {
        <div>
            "Label "
            <input value="some_value" />
        </div>
    };

    let dom2 = VDomElement::build("div")
        .children(vec![
            VDomText::new("Label ").into(),
            VDomElement::build("input")
                .attr("value", "some_value")
                .into()
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );
}

#[test]
fn managed_input() {
    let init_value = "old value".to_string();
    let value = Value::new(init_value.clone());

    let on_input = bind(&value).call_param(|_, value, new: String| {
        value.set(new);
    });

    let input = html! {
        <input value={init_value.as_str()} on_input={on_input} />
    };

    assert_empty(&input, "input");

    let func = input.on_input.unwrap();
    transaction(|context| {
        assert_eq!(value.get(context), "old value");
    });
    assert_eq!(input.attr.get("value").unwrap(), "old value");

    func("new value".to_string());
    transaction(|context| {
        assert_eq!(value.get(context), "new value");
    });
}
