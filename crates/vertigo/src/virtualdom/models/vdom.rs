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
    pub fn node(name: &'static str, attr_list: Vec<NodeAttr>, children: Vec<Self>) -> Self {
        VDomNode::Node {
            node: VDomElement::new(name, attr_list, children)
        }
    }

    pub fn text<T: Into<String>>(value: T) -> Self {
        Self::Text {
            node: VDomText::new(value)
        }
    }

    pub fn component(value: VDomComponent) -> Self {
        Self::Component {
            node: value
        }
    }
}
