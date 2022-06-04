use std::any::Any;

use crate::{
    virtualdom::{
        models::{
            dom_id::DomId,
            dom_node::DomElement,
            vdom_component::VDomComponent
        },
    }, get_driver,
};

/// Starting point of the app.
///
/// Given the driver, the state and main render function, it creates necessary vertigo facilities
/// and runs a never-ending future of reactivity.
pub fn start_app(component: VDomComponent) -> Box<dyn Any> {
    let root = DomElement::create_with_id(DomId::root());
    let subscription = component.render_to(root);
    get_driver().flush_update();

    subscription
}
