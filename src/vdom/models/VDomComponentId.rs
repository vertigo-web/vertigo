use crate::{
    lib::{
        Computed::Computed,
        GraphId::GraphId,
    },
    vdom::{
        models::VDom::VDom,
    }
};
pub struct VDomComponentId {
    idComputed: GraphId,        //id tego computed
    idFunction: u64,            //id tej konkretnej funkcji statycznej (renderującej komponent)
}

impl VDomComponentId {
    pub fn new<T>(params: &Computed<T>, render: fn(&T) -> Vec<VDom>) -> VDomComponentId {

        let idFunction = render as *const () as u64;
        VDomComponentId {
            idComputed: params.getId(),
            idFunction
        }
    }
}
