use crate::virtualdom::models::{
    vdom_component::VDomComponent,
    vdom_element::VDomElement,
    vdom_text::VDomText,
    node_attr::NodeAttr,
};

#[derive(Debug)]
pub enum VDomNode {
    Element {
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
        VDomNode::Element {
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


impl From<VDomComponent> for VDomNode {
    fn from(node: VDomComponent) -> Self {
        Self::Component { node }
    }
}

impl From<VDomElement> for VDomNode {
    fn from(node: VDomElement) -> Self {
        Self::Element { node }
    }
}

impl From<VDomText> for VDomNode {
    fn from(node: VDomText) -> Self {
        Self::Text { node }
    }
}
