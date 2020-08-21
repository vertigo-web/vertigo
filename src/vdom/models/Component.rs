use std::fmt::Debug;
use crate::{
    lib::{
        Computed::Computed,
        GraphId::GraphId,
        Dependencies::Dependencies,
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
    fn new<T>(params: &Computed<T>, render: &(fn(&T) -> Vec<VDom>)) -> ComponentId {
        todo!();
    }
}

pub struct Component {
    id: ComponentId,
    render: Computed<Vec<VDom>>,
}

impl Component {
    pub fn newComponent<T: Debug + 'static>(root: Dependencies, params: Computed<T>, render: fn(&T) -> Vec<VDom>) -> Component {

        let componentId = ComponentId::new(&params, &render);

        let render = params.map(render);

        Component {
            id: componentId,
            render,
        }
    }
}
