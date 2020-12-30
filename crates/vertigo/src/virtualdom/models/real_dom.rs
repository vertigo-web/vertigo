use crate::virtualdom::{
    models::{
        real_dom_node::RealDomNode,
        real_dom_text::RealDomText,
        real_dom_id::RealDomId,
        real_dom_component::RealDomComponent,
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
