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
    pub fn new<T: 'static>(driver: DomDriver, deps: Dependencies, param: T, render: fn(&T) -> Vec<VDom>) -> App {
        let nodeList = RealDomChild::newWithParent(driver, RealDomId::root());

        let render /* (Fn() -> Rc<Vec<VDom>> */ = move || render(&param);
        let vDomComputed: Computed<Vec<VDom>> = deps.from(render);
        let cssManager = CssManager::new();

        let subscription = renderToNode(cssManager.clone(), nodeList, vDomComputed);

        App {
            _subscription: subscription,
            _cssManager: cssManager,
        }
    }
}
