use crate::computed::{
    Dependencies::Dependencies,
    Computed::Computed,
    Client::Client,
};

use crate::vdom::{
    models::{
        VDom::VDom,
        RealDomChild::RealDomChild,
        RealDomId::RealDomId,
        CssManager::CssManager,
    },
    renderToNode::renderToNode,
    DomDriver::{
        DomDriver::DomDriver,
    }
};

pub struct App {
    _subscription: Client,
    _cssManager: CssManager
}

impl App {
    pub fn createRenderComputed<T: 'static>(deps: Dependencies, param: T, render: fn(&T) -> Vec<VDom>) -> Computed<Vec<VDom>> {
        let render /* (Fn() -> Rc<Vec<VDom>> */ = move || render(&param);
        let vDomComputed: Computed<Vec<VDom>> = deps.from(render);

        vDomComputed
    }

    pub fn new(driver: DomDriver, vDomComputed: Computed<Vec<VDom>>) -> App {
        let cssManager = CssManager::new(&driver);
        let nodeList = RealDomChild::newWithParent(driver, RealDomId::root());

        let subscription = renderToNode(cssManager.clone(), nodeList, vDomComputed);

        App {
            _subscription: subscription,
            _cssManager: cssManager,
        }
    }

    pub fn start_app(&self) {
        log::info!("START APP");
    }
}
