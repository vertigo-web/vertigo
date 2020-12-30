use crate::virtualdom::models::{
    v_dom_component::VDomComponent,
    v_dom_node::VDomNode,
    v_dom_text::VDomText,
    node_attr::NodeAttr,
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

    pub fn component(value: VDomComponent) -> VDom {
        VDom::Component {
            node: value
        }
    }
}
