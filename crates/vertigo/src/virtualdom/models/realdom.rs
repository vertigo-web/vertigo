use crate::virtualdom::{
    models::{
        realdom_node::RealDomElement,
        realdom_text::RealDomText,
        realdom_id::RealDomId,
        realdom_component::RealDomComponent,
    },
};

pub enum RealDomNode {
    Node {
        node: RealDomElement,
    },
    Text {
        node: RealDomText,
    },
    Component {
        node: RealDomComponent,
    }
}

impl RealDomNode {
    pub fn id(&self) -> RealDomId {
        match self {
            RealDomNode::Node { node } => {
                node.id_dom()
            },
            RealDomNode::Text { node } => {
                node.id_dom.clone()
            },
            RealDomNode::Component { node } => {
                node.node.id_dom()
            }
        }
    }
}
