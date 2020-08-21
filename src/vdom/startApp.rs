use std::rc::Rc;

use crate::lib::{
    Dependencies::Dependencies,
    Computed::Computed,
    Client::Client,
};

use crate::vdom::{
    models::{
        Handler::{
            Handler, HandlerTarget,
        },
        VDom::VDom,
    },
    renderToNode::renderToNode,
};

//lib
pub fn startApp<T: 'static>(deps: Dependencies, param: T, render: fn(&T) -> Vec<VDom>) -> Client {

    let renderedHandler = Handler::new(HandlerTarget::root());

    let render /* (Fn() -> Rc<Vec<VDom>> */ = move || Rc::new(render(&param));
    let vDomComputed: Computed<Rc<Vec<VDom>>> = deps.from(render);

    let subscription = renderToNode(renderedHandler, vDomComputed);
    subscription
}
