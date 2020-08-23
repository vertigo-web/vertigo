use std::fmt::Debug;
use crate::{
    lib::{
        Computed::Computed,
    },
    vdom::{
        models::{
            VDom::VDom,
            VDomComponentId::VDomComponentId,
        }
    }
};

#[derive(Clone)]
pub struct VDomComponent {
    id: VDomComponentId,
    render: Computed<Vec<VDom>>,
}

impl VDomComponent {
    pub fn new<T: Debug + 'static>(params: Computed<T>, render: fn(&T) -> Vec<VDom>) -> VDomComponent {

        let componentId = VDomComponentId::new(&params, render);
        let render = params.map(render);

        VDomComponent {
            id: componentId,
            render,
        }
    }
}
