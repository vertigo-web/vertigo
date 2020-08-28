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
    }
};


pub fn startApp<T: 'static>(driver: DomDriver, deps: Dependencies, param: T, render: fn(&T) -> Vec<VDom>) -> Client {
    let nodeList = RealDomChild::newWithParent(driver, RealDomId::root());

    let render /* (Fn() -> Rc<Vec<VDom>> */ = move || render(&param);
    let vDomComputed: Computed<Vec<VDom>> = deps.from(render);

    let subscription = renderToNode(nodeList, vDomComputed);
    subscription
}
