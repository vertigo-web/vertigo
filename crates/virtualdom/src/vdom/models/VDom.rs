use crate::vdom::models::{
    VDomComponent::VDomComponent,
    VDomNode::VDomNode,
    VDomText::VDomText,
    NodeAttr::NodeAttr,
};

pub enum VDom {
    Node {
        node: VDomNode,
    },
    Text {
        node: VDomText,
    },
    Component {
        node: VDomComponent,
    },
}

impl VDom {
    pub fn node(name: &'static str, childList: Vec<NodeAttr>) -> VDom {
        VDom::Node {
            node: VDomNode::new(name, childList)
        }
    }

    pub fn text<T: Into<String>>(value: T) -> VDom {
        VDom::Text {
            node: VDomText::new(value)
        }
    }
}
