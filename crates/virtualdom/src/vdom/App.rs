use crate::computed::{
    Client::Client,
};

use crate::vdom::{
    models::{
        RealDomNode::RealDomNode,
        RealDomId::RealDomId,
        CssManager::CssManager,
        VDomComponent::VDomComponent,
    },
    renderToNode::renderToNode,
    DomDriver::{
        DomDriver::DomDriver,
    }
};

use super::DomDriver::DomDriver::DomDriverTrait;

pub struct App {
    _subscription: Client,
    _cssManager: CssManager
}

impl App {
    pub fn new<D: DomDriverTrait + 'static>(driverIn: D, computed: VDomComponent) -> App {
        let driver = DomDriver::new(driverIn);

        let cssManager = CssManager::new(&driver);
        let nodeList = RealDomNode::createWithId(driver, RealDomId::root());

        let subscription = renderToNode(cssManager.clone(), nodeList, computed);

        App {
            _subscription: subscription,
            _cssManager: cssManager,
        }
    }

    pub fn start_app(&self) {
        log::info!("START APP");
    }
}
