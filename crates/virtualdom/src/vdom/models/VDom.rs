use crate::vdom::models::{
    VDomComponent::VDomComponent,
    VDomNode::VDomNode,
    VDomText::VDomText,
};
use std::collections::HashMap;

#[derive(Clone)]
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
    pub fn node<T: Into<String>>(name: T, attr: HashMap<String, String>, child: Vec<VDom>) -> VDom {
        VDom::Node {
            node: VDomNode {
                name: name.into(),
                attr,
                child
            }
        }
    }

    pub fn text<T: Into<String>>(value: T) -> VDom {
        VDom::Text {
            node: VDomText {
                value: value.into()
            }
        }
    }
}
