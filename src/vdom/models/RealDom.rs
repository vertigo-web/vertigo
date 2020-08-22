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
    },
    DomDriver::{
        DomDriver::DomDriver,
    },
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
    domDriver: DomDriver,
    idDom: RealDomNodeId,
    name: String,
    attr: HashMap<String, String>,
    child: Vec<RealDom>,
}

impl RealDomNode {
    pub fn new(domDriver: DomDriver, name: String) -> RealDomNode {
        let id = RealDomNodeId::new();

        domDriver.createNode(id.clone(), &name);

        let node = RealDomNode {
            domDriver,
            idDom: id,
            name,
            attr: HashMap::new(),
            child: Vec::new(),
        };


        node
    }

    pub fn setAttr(&mut self, name: String, value: String) {
        let needUpdate = {
            let item = self.attr.get(&name);
            if let Some(item) = item {
                if *item == value {
                    false
                } else {
                    true
                }
            } else {
                true
            }
        };

        if needUpdate {
            self.domDriver.setAttr(self.idDom.clone(), &name, &value);
             self.attr.insert(name, value);
       }
    }

    pub fn removeAttr(&mut self, name: String) {
        let needDelete = {
            self.attr.contains_key(&name)
        };

        if needDelete {
            self.attr.remove(&name);
            self.domDriver.removeAttr(self.idDom.clone(), &name);
        }
    }
}

impl Drop for RealDomNode {
    fn drop(&mut self) {
        self.domDriver.remove(self.idDom.clone());
    }
}

pub struct RealDomText {
    domDriver: DomDriver,
    idDom: RealDomNodeId,
    value: String,
}

impl RealDomText {
    pub fn new(domDriver: DomDriver, value: String) -> RealDomText {
        let id = RealDomNodeId::new();

        domDriver.createText(id.clone(), &value);

        let node = RealDomText {
            domDriver,
            idDom: id,
            value,
        };

        node
    }
}

impl Drop for RealDomText {
    fn drop(&mut self) {
        self.domDriver.remove(self.idDom.clone());
    }
}

pub enum RealDom {
    Node {
        node: RealDomNode,
    },
    Text {
        node: RealDomText,
    },
    Component {
        domDriver: DomDriver,
        id: ComponentId,                        //do porównywania
        subscription: Client,                   //Subskrybcją, , wstawia do handler
        handler: Handler,
    }
}

impl RealDom {
    pub fn newNode(domDriver: DomDriver, name: String) -> RealDom {
        RealDom::Node {
            node: RealDomNode::new(domDriver, name)
        }
    }

    pub fn newText(domDriver: DomDriver, value: String) -> RealDom {
        RealDom::Text {
            node: RealDomText::new(domDriver, value)
        }
    }
}
