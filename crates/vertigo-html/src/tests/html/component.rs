use vertigo::{
    computed::{Computed, Dependencies, Value},
    VDomElement,
};

use crate::html;

// Make crate available by its name for html macro
use crate as vertigo_html;

use super::utils::*;

#[test]
fn div_with_text() {
    let deps = Dependencies::default();

    let value = Value::new(deps, "old value".to_string());

    fn my_component(state: &Computed<String>) -> VDomElement {
        html!("
            <div>Value {state.get_value().as_str()}</div>
        ")
    }

    let div = html!("
        <div>
            <component {my_component} data={value.to_computed()} />
        </div>
    ");

    assert_eq!(div.name, "div");
    assert_eq!(div.children.len(), 1);

    let comp = get_component(&div.children[0]);
    let inner_div = comp.view.get_value();
    assert_eq!(inner_div.name, "div");
    assert_eq!(inner_div.children.len(), 2);

    let label = get_text(&inner_div.children[0]);
    assert_eq!(label.value, "Value");
    let expr = get_text(&inner_div.children[1]);
    assert_eq!(expr.value, "old value");

    value.set_value("new value".to_string());

    // Get the component again after changing state
    let comp = get_component(&div.children[0]);
    let inner_div = comp.view.get_value();
    let expr = get_text(&inner_div.children[1]);
    assert_eq!(expr.value, "new value");
}
