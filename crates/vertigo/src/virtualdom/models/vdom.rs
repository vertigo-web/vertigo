use crate::virtualdom::models::{
    vdom_component::VDomComponent,
    vdom_node::VDomElement,
    vdom_text::VDomText,
    node_attr::NodeAttr,
};

pub enum VDomNode {
    Node {
        node: VDomElement,
    },
    Text {
        node: VDomText,
    },
    Component {
        node: VDomComponent,
    },
}

impl VDomNode {
    pub fn node(name: &'static str, child_list: Vec<NodeAttr>) -> VDomNode {
        VDomNode::Node {
            node: VDomElement::new(name, child_list)
        }
    }

    pub fn text<T: Into<String>>(value: T) -> VDomNode {
        VDomNode::Text {
            node: VDomText::new(value)
        }
    }

    pub fn component(value: VDomComponent) -> VDomNode {
        VDomNode::Component {
            node: value
        }
    }
}
