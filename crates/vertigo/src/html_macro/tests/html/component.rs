use crate::{
    computed::{Computed, Dependencies, Value},
    VDomComponent,
    VDomElement, html,
};

use super::utils::*;

// Make crate available by its name for html macro
use crate as vertigo;

#[test]
fn div_with_component() {
    let deps = Dependencies::default();

    let value = Value::new(deps, "old value".to_string());
    let value_computed = value.to_computed();

    fn my_component(state: &Computed<String>) -> VDomElement {
        html! {
            <div>"Value "{state.get_value().as_str()}</div>
        }
    }

    let dom1 = html! {
        <div>
            <component {my_component} data={value_computed} />
        </div>
    };

    let dom2 = VDomElement::build("div")
        .children(vec![
            VDomComponent::new(value_computed, my_component).into()
        ]);

    assert_eq!(
        format!("{:?}", dom1),
        format!("{:?}", dom2)
    );

    // Check if computed value changes

    let comp = get_component(&dom1.children[0]);
    let inner_div = comp.view.get_value();
    let expr = get_text(&inner_div.children[1]);
    assert_eq!(expr.value, "old value");

    value.set_value("new value".to_string());

    // Get the component again after changing state
    let comp = get_component(&dom1.children[0]);
    let inner_div = comp.view.get_value();
    let expr = get_text(&inner_div.children[1]);
    assert_eq!(expr.value, "new value");
}
