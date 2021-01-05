use crate::virtualdom::models::{
    vdom_component::VDomComponent,
    vdom_node::VDomNode,
    vdom_text::VDomText,
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
    pub fn node(name: &'static str, child_list: Vec<NodeAttr>) -> VDom {
        VDom::Node {
            node: VDomNode::new(name, child_list)
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
