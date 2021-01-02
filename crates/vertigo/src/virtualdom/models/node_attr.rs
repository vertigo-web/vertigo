use alloc::rc::Rc;
use core::cmp::PartialEq;
use alloc::{
    string::String,
    vec::Vec
};

use crate::virtualdom::models::{
    v_dom::VDom,
    v_dom_node::VDomNode,
    v_dom_component::VDomComponent,
};
use crate::virtualdom::models::css::Css;
use crate::computed::{
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
    onMouseEnter {
        event: Rc<dyn Fn()>
    },
    onMouseLeave {
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

pub fn onClick<F: Fn() + 'static>(callback: F) -> NodeAttr {
    NodeAttr::OnClick {
        event: Rc::new(callback),
    }
}

pub fn onInput<F: Fn(String) + 'static>(callback: F) -> NodeAttr {
    NodeAttr::OnInput {
        event: Rc::new(callback),
    }
}

pub fn onMouseEnter<F: Fn() + 'static>(callback: F) -> NodeAttr {
    NodeAttr::onMouseEnter {
        event: Rc::new(callback),
    }
}

pub fn onMouseLeave<F: Fn() + 'static>(callback: F) -> NodeAttr {
    NodeAttr::onMouseLeave {
        event: Rc::new(callback),
    }
}

pub fn attr<K: Into<String>>(name: &'static str, value: K) -> NodeAttr {
    NodeAttr::Attr {
        name,
        value: value.into()
    }
}

pub fn node(name: &'static str, childList: Vec<NodeAttr>) -> NodeAttr {
    NodeAttr::Node {
        node: VDom::node(name, childList)
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

pub fn buildNode(name: &'static str, childList: Vec<NodeAttr>) -> VDomNode {
    VDomNode::new(name, childList)
}

pub fn buildText<T: Into<String>>(name: T) -> VDom {
    VDom::text(name)
}
