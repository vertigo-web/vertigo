use vertigo::{VDomNode, VDomText, VDomComponent, VDomElement};

pub fn assert_empty(el: &VDomElement, name: &str) {
    assert_eq!(el.name, name);
    assert_eq!(el.children.len(), 0);
}

// pub fn get_node(node: &VDomNode) -> &VDomElement {
//     match node {
//         VDomNode::Element { node } => node,
//         VDomNode::Text { .. } => panic!("get_node: expected Node, got Text"),
//         VDomNode::Component { .. } => panic!("get_node: expected Node, got Component"),
//     }
// }

pub fn get_text(node: &VDomNode) -> &VDomText {
    match node {
        VDomNode::Element { .. } => panic!("get_text: Expected text, got Node"),
        VDomNode::Text { node } => node,
        VDomNode::Component { .. } => panic!("get_text: Expected text, got Component"),
    }
}

pub fn get_component(node: &VDomNode) -> &VDomComponent {
    match node {
        VDomNode::Element { .. } => panic!("get_component: Expected Component, got Node"),
        VDomNode::Text { .. } => panic!("get_component: Expected Component, got Text"),
        VDomNode::Component { node } => node,
    }
}
