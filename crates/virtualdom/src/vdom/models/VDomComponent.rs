use std::rc::Rc;

use crate::computed::{
    Computed::Computed,
};

use crate::{
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
    pub view: Computed<VDomNode>,
}

impl VDomComponent {
    pub fn new<T: 'static>(params: Computed<T>, render: fn(&Computed<T>) -> VDomNode) -> VDomComponent {

        let componentId = VDomComponentId::new(&params, render);
        let view = params.map_for_render(render);

        VDomComponent {
            id: componentId,
            view,
        }
    }
}
