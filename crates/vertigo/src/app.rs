use crate::computed::{
    Client,
};

use crate::{
    virtualdom::{
        models::{
            realdom_node::RealDomElement,
            realdom_id::RealDomId,
            vdom_component::VDomComponent,
        },
        render_to_node::render_to_node,
    },
    css_manager::css_manager::CssManager,
    driver::DomDriver,
};

pub struct App {
    _subscription: Client,
    _css_manager: CssManager
}

impl App {
    pub fn new(driver: DomDriver, computed: VDomComponent) -> App {
        let css_manager = CssManager::new(&driver);
        let root = RealDomElement::create_with_id(driver, RealDomId::root());

        let subscription = render_to_node(css_manager.clone(), root, computed);

        App {
            _subscription: subscription,
            _css_manager: css_manager,
        }
    }

    pub async fn start_app(&self) {
        log::info!("START APP");

        let wait = std::future::pending();
        wait.await
    }
}
