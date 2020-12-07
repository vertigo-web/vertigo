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
            VDom::VDom,
            VDomComponentId::VDomComponentId,
        }
    }
};

#[derive(Clone)]
pub struct VDomComponent {
    pub id: VDomComponentId,
    render: Computed<Vec<VDom>>,
}

impl VDomComponent {
    pub fn new<T: 'static>(params: Computed<T>, render: fn(Rc<T>) -> Vec<VDom>) -> VDomComponent {

        let componentId = VDomComponentId::new(&params, render);
        let render = params.map(render);

        VDomComponent {
            id: componentId,
            render,
        }
    }

    pub fn subscribe<F: Fn(&Vec<VDom>) + 'static>(self, render: F) -> Client {
        self.render.subscribe(render)
    }
}
