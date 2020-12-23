use crate::virtualdom::{
    models::{
        RealDomNode::RealDomNode,
        RealDomText::RealDomText,
        RealDomId::RealDomId,
        RealDomComponent::RealDomComponent,
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
    pub fn id(&self) -> RealDomId {
        match self {
            RealDom::Node { node } => {
                node.idDom()
            },
            RealDom::Text { node } => {
                node.idDom.clone()
            },
            RealDom::Component { node } => {
                node.node.idDom()
            }
        }
    }
}
