use core::cmp::PartialEq;
use crate::computed::{
    Computed,
};

use crate::{
    virtualdom::{
        models::{
            v_dom_node::VDomNode,
            v_dom_component_id::VDomComponentId,
        }
    }
};

#[derive(Clone)]
pub struct VDomComponent {
    pub id: VDomComponentId,
    pub view: Computed<VDomNode>,
}

impl VDomComponent {
    pub fn new<T: PartialEq + 'static>(params: Computed<T>, render: fn(&Computed<T>) -> VDomNode) -> VDomComponent {

        let componentId = VDomComponentId::new(&params, render);
        let view = params.map_for_render(render);

        VDomComponent {
            id: componentId,
            view,
        }
    }
}
