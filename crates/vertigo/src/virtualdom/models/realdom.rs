use crate::virtualdom::models::{
    realdom_component::RealDomComponent,
    realdom_node::RealDomElement,
    realdom_text::RealDomText,
};

pub enum RealDomNode {
    Node { node: RealDomElement },
    Text { node: RealDomText },
    Component { node: RealDomComponent },
}

impl RealDomNode {
    pub fn new_node(node: RealDomElement) -> RealDomNode {
        RealDomNode::Node { node }
    }

    pub fn new_text(node: RealDomText) -> RealDomNode {
        RealDomNode::Text { node }
    }

    pub fn new_component(node: RealDomComponent) -> RealDomNode {
        RealDomNode::Component { node }
    }
}
