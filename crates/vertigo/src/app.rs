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

#[cfg(feature = "wasm_logger")]
use crate::{Computed, VDomElement};

/// Starting point of the app.
///
/// Given the driver, the state and main render function, it creates necessary vertigo facilities
/// and runs a never-ending future of reactivity.
pub async fn start_app<T: PartialEq + 'static>(driver: Driver, app_state: Computed<T>, render: fn(&Computed<T>) -> VDomElement) {
    #[cfg(feature = "wasm_logger")]
    {
        console_error_panic_hook::set_once();
        wasm_logger::init(wasm_logger::Config::default());
    }

    let component = VDomComponent::new(app_state, render);
    let css_manager = CssManager::new(&driver);
    let root = RealDomElement::create_with_id(driver.clone(), RealDomId::root());

    let _subscription = render_to_node(driver.clone(), css_manager.clone(), root, component);
    driver.flush_update();

    std::future::pending::<()>().await
}
