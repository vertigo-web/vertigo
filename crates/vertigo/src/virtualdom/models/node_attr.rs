use std::rc::Rc;
use std::cmp::PartialEq;

use crate::virtualdom::models::{
    vdom::VDomNode,
    vdom_node::VDomElement,
    vdom_component::VDomComponent,
};
use crate::virtualdom::models::css::Css;
use crate::computed::{
    Value,
    Computed,
};

pub enum NodeAttr {
    Css {
        css: Css
    },
    OnClick {
        event: Rc<dyn Fn()>
    },
    OnInput {
        event: Rc<dyn Fn(String)>
    },
    OnMouseEnter {
        event: Rc<dyn Fn()>
    },
    OnMouseLeave {
        event: Rc<dyn Fn()>
    },
    Attr {
        name: &'static str,
        value: String,
    },
    Node {
        node: VDomNode,
    }
}



pub fn css(css: Css) -> NodeAttr {
    NodeAttr::Css {
        css,
    }
}

pub fn on_click<F: Fn() + 'static>(callback: F) -> NodeAttr {
    NodeAttr::OnClick {
        event: Rc::new(callback),
    }
}

pub fn on_input<F: Fn(String) + 'static>(callback: F) -> NodeAttr {
    NodeAttr::OnInput {
        event: Rc::new(callback),
    }
}

pub fn on_mouse_enter<F: Fn() + 'static>(callback: F) -> NodeAttr {
    NodeAttr::OnMouseEnter {
        event: Rc::new(callback),
    }
}

pub fn on_mouse_leave<F: Fn() + 'static>(callback: F) -> NodeAttr {
    NodeAttr::OnMouseLeave {
        event: Rc::new(callback),
    }
}

pub fn attr<K: Into<String>>(name: &'static str, value: K) -> NodeAttr {
    NodeAttr::Attr {
        name,
        value: value.into()
    }
}

pub fn node(name: &'static str, child_list: Vec<NodeAttr>) -> NodeAttr {
    NodeAttr::Node {
        node: VDomNode::node(name, child_list)
    }
}

pub fn text<T: Into<String>>(name: T) -> NodeAttr {
    NodeAttr::Node {
        node: VDomNode::text(name)
    }
}

pub fn component<T: PartialEq + 'static>(params: Computed<T>, render: fn(&Computed<T>) -> VDomElement) -> NodeAttr {
    NodeAttr::Node {
        node:VDomNode::component(VDomComponent::new(params, render))
    }
}

pub fn component_value<T: PartialEq + 'static>(params: Value<T>, render: fn(&Value<T>) -> VDomElement) -> NodeAttr {
    NodeAttr::Node {
        node:VDomNode::component(VDomComponent::from_value(params, render))
    }
}

pub fn build_node(name: &'static str, child_list: Vec<NodeAttr>) -> VDomElement {
    VDomElement::new(name, child_list)
}

pub fn build_text<T: Into<String>>(name: T) -> VDomNode {
    VDomNode::text(name)
}
