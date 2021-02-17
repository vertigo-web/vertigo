use vertigo::{VDomNode, VDomText, VDomComponent, VDomElement, CssGroup};

pub fn assert_empty(el: &VDomElement, name: &str) {
    assert_eq!(el.name, name);
    assert_eq!(el.child.len(), 0);
}

pub fn get_node(node: &VDomNode) -> &VDomElement {
    match node {
        VDomNode::Element { node } => node,
        VDomNode::Text { .. } => panic!("get_node: expected Node, got Text"),
        VDomNode::Component { .. } => panic!("get_node: expected Node, got Component"),
    }
}

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

pub fn get_static_css(group: &CssGroup) -> &'static str {
    match group {
        CssGroup::CssStatic { value } => value,
        CssGroup::CssDynamic { .. } => panic!("get_static_css: Expected CssStatic, got CssDynamic"),
    }
}

pub fn get_dynamic_css(group: &CssGroup) -> &String {
    match group {
        CssGroup::CssStatic { .. } => panic!("get_static_css: Expected CssDynamic, got CssStatic"),
        CssGroup::CssDynamic { value } => value,
    }
}
