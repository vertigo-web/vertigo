use crate::virtualdom::{
    models::{
        realdom_node::RealDomNode,
        realdom_text::RealDomText,
        realdom_id::RealDomId,
        realdom_component::RealDomComponent,
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
                node.id_dom()
            },
            RealDom::Text { node } => {
                node.id_dom.clone()
            },
            RealDom::Component { node } => {
                node.node.id_dom()
            }
        }
    }
}
