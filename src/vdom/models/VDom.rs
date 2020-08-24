use crate::vdom::models::{
    VDomComponent::VDomComponent,
    VDomNode::VDomNode,
    VDomText::VDomText,
};

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
