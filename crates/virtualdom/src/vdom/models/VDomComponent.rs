use std::rc::Rc;

use crate::computed::{
    Computed::Computed,
};

use crate::{
    computed::{
        Client::Client
    },
    vdom::{
        models::{
            VDomNode::VDomNode,
            VDomComponentId::VDomComponentId,
        }
    }
};

#[derive(Clone)]
pub struct VDomComponent {
    pub id: VDomComponentId,
    render: Computed<VDomNode>,
}

impl VDomComponent {
    pub fn new<T: 'static>(params: Computed<T>, render: fn(Rc<T>) -> VDomNode) -> VDomComponent {

        let componentId = VDomComponentId::new(&params, render);
        let render = params.map(render);

        VDomComponent {
            id: componentId,
            render,
        }
    }

    pub fn subscribe<F: Fn(&VDomNode) + 'static>(self, render: F) -> Client {
        self.render.subscribe(render)
    }
}
