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
    },
    renderToNode::renderToNode,
    DomDriver::{
        DomDriver::DomDriver,
        DomDriverPrint::DomDriverPrint,
    }
};


pub fn startApp<T: 'static>(deps: Dependencies, param: T, render: fn(&T) -> Vec<VDom>) -> Client {

    let driverPrint = DomDriverPrint::new();
    let driver = DomDriver::new(driverPrint);
    
    let nodeList = RealDomChild::newWithParent(driver, RealDomId::root());

    let render /* (Fn() -> Rc<Vec<VDom>> */ = move || render(&param);
    let vDomComputed: Computed<Vec<VDom>> = deps.from(render);

    let subscription = renderToNode(nodeList, vDomComputed);
    subscription
}
