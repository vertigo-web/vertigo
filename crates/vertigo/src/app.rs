use crate::{
    virtualdom::{
        models::{
            realdom_node::RealDomElement,
            realdom_id::RealDomId,
            vdom_component::VDomComponent,
        },
        render_to_node::render_to_node,
    },
    css::css_manager::CssManager,
    driver::Driver,
};

pub async fn start_app(driver: Driver, computed: VDomComponent) {
    let css_manager = CssManager::new(&driver);
    let root = RealDomElement::create_with_id(driver, RealDomId::root());

    let subscription = render_to_node(css_manager.clone(), root, computed);

    std::future::pending::<()>().await;

    subscription.off();
}
