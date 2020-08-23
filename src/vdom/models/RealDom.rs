use crate::lib::{
    Client::Client,
};

use crate::vdom::{
    models::{
        RealDomNode::RealDomNode,
        RealDomText::RealDomText,
        RealDomChild::RealDomChild,
        VDomComponentId::VDomComponentId,
        RealDomId::RealDomId,
    },
    DomDriver::{
        DomDriver::DomDriver,
    },
};


pub enum RealDom {
    Node {
        node: RealDomNode,
    },
    Text {
        node: RealDomText,
    },
    Component {
        id: VDomComponentId,                    //do porównywania
        subscription: Client,                   //Subskrybcją, , wstawia do handler
        child: RealDomChild,
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

    pub fn firstChildId(&self) -> RealDomId {
        match self {
            RealDom::Node { node } => {
                node.idDom.clone()
            },
            RealDom::Text { node } => {
                node.idDom.clone()
            },
            RealDom::Component { child, .. } => {
                child.firstChildId()
            }
        }
    }

    pub fn lastChildId(&self) -> RealDomId {
        match self {
            RealDom::Node { node } => {
                node.idDom.clone()
            },
            RealDom::Text { node } => {
                node.idDom.clone()
            },
            RealDom::Component { child, .. } => {
                child.lastChildId()
            }
        }
    }

    pub fn childIds(&self) -> Vec<RealDomId> {
        match self {
            RealDom::Node { node } => {
                vec!(node.idDom.clone())
            },
            RealDom::Text { node } => {
                vec!(node.idDom.clone())
            },
            RealDom::Component { child, .. } => {
                child.childIds()
            }
        }
    }
}
