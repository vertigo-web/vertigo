use crate::computed::{
    Computed,
};

use crate::{
    virtualdom::{
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