use crate::computed::{
    Client,
};

use crate::{
    vdom::{
        models::{
            RealDomNode::RealDomNode,
            RealDomId::RealDomId,
            CssManager::CssManager,
            VDomComponent::VDomComponent,
        },
        renderToNode::renderToNode,
    },
    driver::DomDriver,
};

pub struct App {
    _subscription: Client,
    _cssManager: CssManager
}

impl App {
    pub fn new(driver: DomDriver, computed: VDomComponent) -> App {
        let cssManager = CssManager::new(&driver);
        let root = RealDomNode::createWithId(driver, RealDomId::root());

        let subscription = renderToNode(cssManager.clone(), root, computed);

        App {
            _subscription: subscription,
            _cssManager: cssManager,
        }
    }

    pub fn start_app(&self) {
        log::info!("START APP");
    }
}