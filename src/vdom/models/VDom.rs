use std::collections::HashMap;

use crate::vdom::models::VDomComponent::VDomComponent;

pub struct VDomNode {
    name: String,
    attr: HashMap<String, String>,
    child: Vec<VDom>,
}


pub enum VDom {
    Node {
        node: VDomNode,
    },
    Text {
        value: String,
    },
    Component {
        node: VDomComponent,
    },
}
