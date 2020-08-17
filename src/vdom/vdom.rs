use std::collections::HashMap;
use crate::lib::GraphId::GraphId;

struct ComponentId {
    idComputed: GraphId,        //id tego computed
    idFunction: u64,            //id tej konkretnej funkcji statycznej (renderującej komponent)
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
