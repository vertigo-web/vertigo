use vertigo::computed::{Dependencies, Value};

use crate::html;

use super::utils::*;

#[test]
fn div_with_label_and_input() {
    let div = html!(r#"
        <div>
            Label
            <input value="some_value" />
        </div>
    "#);

    assert_eq!(div.name, "div");
    assert_eq!(div.child.len(), 2);

    let label = get_text(&div.child[0]);
    assert_eq!(label.value, "Label");

    let input = get_node(&div.child[1]);
    assert_eq!(input.name, "input");
    assert_eq!(input.attr.get("value"), Some(&"some_value".to_string()));
}

#[test]
fn managed_input() {
    let value = Value::new(Dependencies::default(), "old value".to_string());

    let on_input = {
        let value = value.clone();
        move |new: String| {
            value.set_value(new);
        }
    };

    let input = html!("
        <input value={value.get_value().as_str()} onInput={on_input} />
    ");

    assert_empty(&input, "input");

    let func = input.on_input.unwrap();
    assert_eq!(*value.get_value(), "old value");
    assert_eq!(input.attr.get("value").unwrap(), "old value");

    func("new value".to_string());
    assert_eq!(*value.get_value(), "new value");
}
