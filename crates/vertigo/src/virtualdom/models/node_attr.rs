use std::rc::Rc;
use std::cmp::PartialEq;

use crate::virtualdom::models::{
    vdom::VDom,
    vdom_node::VDomNode,
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
        node: VDom,
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
        node: VDom::node(name, child_list)
    }
}

pub fn text<T: Into<String>>(name: T) -> NodeAttr {
    NodeAttr::Node {
        node: VDom::text(name)
    }
}

pub fn component<T: PartialEq + 'static>(params: Computed<T>, render: fn(&Computed<T>) -> VDomNode) -> NodeAttr {
    NodeAttr::Node {
        node:VDom::component(VDomComponent::new(params, render))
    }
}

pub fn component_value<T: PartialEq + 'static>(params: Value<T>, render: fn(&Value<T>) -> VDomNode) -> NodeAttr {
    NodeAttr::Node {
        node:VDom::component(VDomComponent::from_value(params, render))
    }
}

pub fn build_node(name: &'static str, child_list: Vec<NodeAttr>) -> VDomNode {
    VDomNode::new(name, child_list)
}

pub fn build_text<T: Into<String>>(name: T) -> VDom {
    VDom::text(name)
}
