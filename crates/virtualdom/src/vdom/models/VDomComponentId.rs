use std::rc::Rc;
use crate::computed::{
    Computed::Computed,
    GraphId::GraphId,
};

use crate::{
    vdom::{
        models::VDom::VDom,
    }
};

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct VDomComponentId {
    idComputed: GraphId,        //id tego computed
    idFunction: u64,            //id tej konkretnej funkcji statycznej (renderującej komponent)
}

impl VDomComponentId {
    pub fn new<T>(params: &Computed<T>, render: fn(Rc<T>) -> Vec<VDom>) -> VDomComponentId {

        let idFunction = render as *const () as u64;
        VDomComponentId {
            idComputed: params.getId(),
            idFunction
        }
    }
}
