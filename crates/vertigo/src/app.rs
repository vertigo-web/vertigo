use crate::computed::{
    Client,
};

use crate::{
    virtualdom::{
        models::{
            RealDomNode::RealDomNode,
            RealDomId::RealDomId,
            VDomComponent::VDomComponent,
        },
        renderToNode::renderToNode,
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
        let root = RealDomNode::createWithId(driver, RealDomId::root());

        let subscription = renderToNode(css_manager.clone(), root, computed);

        App {
            _subscription: subscription,
            _css_manager: css_manager,
        }
    }

    pub fn start_app(&self) {
        log::info!("START APP");
    }
}
