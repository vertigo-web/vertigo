use crate::{
    css::css_manager::CssManager,
    Computed,
    computed::Client,
    driver::Driver,
    VDomElement,
    virtualdom::{
        models::{
            realdom_id::RealDomId,
            realdom_node::RealDomElement,
            vdom_component::VDomComponent
        },
        render_to_node::render_to_node,
    },
};

/// Starting point of the app.
///
/// Given the driver, the state and main render function, it creates necessary vertigo facilities
/// and runs a never-ending future of reactivity.
pub fn start_app<T: PartialEq + 'static>(driver: Driver, app_state: Computed<T>, render: fn(&Computed<T>) -> VDomElement) -> Client {
    let component = VDomComponent::new(app_state, render);
    let css_manager = CssManager::new(&driver);
    let root = RealDomElement::create_with_id(driver.clone(), RealDomId::root());

    let subscription = render_to_node(driver.clone(), css_manager, root, component);
    driver.flush_update();

    subscription
}
