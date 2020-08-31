use crate::vdom::{
    models::{
        RealDomNode::RealDomNode,
        RealDomText::RealDomText,
        RealDomId::RealDomId,
        RealDomComponent::RealDomComponent,
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
        node: RealDomComponent,
    }
}

impl RealDom {
    pub fn newNode(domDriver: DomDriver, name: &'static str) -> RealDom {
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
            RealDom::Component { node } => {
                node.firstChildId()
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
            RealDom::Component { node } => {
                node.lastChildId()
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
            RealDom::Component { node } => {
                node.childIds()
            }
        }
    }
}
