use std::fmt::Debug;
use crate::{
    lib::{
        Computed::Computed,
        GraphId::GraphId,
    },
    vdom::{
        models::VDom::VDom,
    }
};

pub struct ComponentId {
    idComputed: GraphId,        //id tego computed
    idFunction: u64,            //id tej konkretnej funkcji statycznej (renderującej komponent)
}

impl ComponentId {
    fn new<T>(params: &Computed<T>, render: fn(&T) -> Vec<VDom>) -> ComponentId {

        let idFunction = render as *const () as u64;
        ComponentId {
            idComputed: params.getId(),
            idFunction
        }
    }
}

pub struct Component {
    id: ComponentId,
    render: Computed<Vec<VDom>>,
}

impl Component {
    pub fn new<T: Debug + 'static>(params: Computed<T>, render: fn(&T) -> Vec<VDom>) -> Component {

        let componentId = ComponentId::new(&params, render);
        let render = params.map(render);

        Component {
            id: componentId,
            render,
        }
    }
}
