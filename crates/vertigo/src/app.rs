use crate::{
    css::css_manager::CssManager,
    driver::Driver,
    virtualdom::{
        models::{
            realdom_id::RealDomId,
            realdom_node::RealDomElement,
            vdom_component::VDomComponent
        },
        render_to_node::render_to_node,
    },
};

/// Starting point of the app. Given the driver and root component of the app,
/// it creates necessary vertigo facilities and runs a never-ending future of reactivity.
pub async fn start_app(driver: Driver, computed: VDomComponent) {
    let css_manager = CssManager::new(&driver);
    let root = RealDomElement::create_with_id(driver.clone(), RealDomId::root());

    let _subscription = render_to_node(driver.clone(), css_manager.clone(), root, computed);
    driver.flush_update();

    std::future::pending::<()>().await
}
