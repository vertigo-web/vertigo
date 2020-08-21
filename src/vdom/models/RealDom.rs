use std::collections::HashMap;

use crate::lib::{
    Client::Client,
};

use crate::vdom::{
    models::{
        Handler::Handler,
        Component::{
            ComponentId,
        }
    }
};

pub struct RealDomNode {
    name: String,
    attr: HashMap<String, String>,
    child: Vec<RealDom>,
    idDom: u64,                             //id realnego doma
}

pub enum RealDom {
    Node {
        node: RealDomNode,
    },
    Text {
        value: String,
        idDom: u64,                             //id realnego doma
    },
    Component {
        id: ComponentId,                        //do porównywania
        subscription: Client,                   //Subskrybcją, kryje się funkcja, która odpalana (na zmianę), wstawia coś do pojemnika child
                                                //idParenta mozemy przekazywac w momencie gdy będziemy tworzyć Client-a

        handler: Handler,


//     idParent: RealNodeId,               //prawdopodobnie będzie konieczne. Ale ten id moze byc utworzony przy stworzeniu noda. Nie będzie zmieniany.
    }
}
