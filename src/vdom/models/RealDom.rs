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

const ROOT_ID: u64 = 1;
const START_ID: u64 = 2;

fn get_unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER:AtomicU64 = AtomicU64::new(START_ID);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone, Debug)]
pub struct RealDomNodeId {
    id: u64,
}

impl RealDomNodeId {
    pub fn root() -> RealDomNodeId {
        RealDomNodeId {
            id: ROOT_ID
        }
    }

    pub fn new() -> RealDomNodeId {
        RealDomNodeId {
            id: get_unique_id()
        }
    }
}

impl std::fmt::Display for RealDomNodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "RealDomNodeId={}", self.id)
    }
}


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
