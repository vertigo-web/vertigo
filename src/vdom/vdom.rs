use std::fmt::Debug;
use std::collections::HashMap;
use crate::lib::{
    GraphId::GraphId,
    Computed::Computed,
    Dependencies::Dependencies,
};

struct ComponentId {
    idComputed: GraphId,        //id tego computed
    idFunction: u64,            //id tej konkretnej funkcji statycznej (renderującej komponent)
}

struct Component {
    id: ComponentId,
    render: Box<dyn Fn() -> Vec<VDom>>
}

fn newComponent<T: Debug>(root: Dependencies, params: Computed<T>, render: fn(T) -> Vec<VDom>) -> Component {
    let clientId = 4;   //TODO
    //let getValue = root.wrapGetValue(render, clientId);

    // to trzeba zamienic na subksrybcje
    // trzeba wystawic jakas funkcje subskryubujaca na funkcje (autorun)

    todo!();
}

enum VDom {
    Node {
        name: String,
        attr: HashMap<String, String>
    },

    Text {
        value: String,
    },

    Component {
        id: ComponentId,
        render: fn() -> Vec<VDom>
    }
}

enum RealDom {
    Node {
        name: String,
        attr: HashMap<String, String>,
    },
    Text {
        value: String,
    },
    Component {
        id: ComponentId,
    }
}
