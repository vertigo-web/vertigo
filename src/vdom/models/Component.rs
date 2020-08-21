use crate::{
    lib::{
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


pub struct Component {
    id: ComponentId,
    render: Box<dyn Fn(&Dependencies) -> Vec<VDom>>
}
