use std::collections::HashMap;
use std::rc::Rc;
use std::cmp::PartialEq;

use crate::virtualdom::models::{
    vdom::VDomNode,
};
use crate::virtualdom::models::{
    css::Css,
    node_attr::NodeAttr,
};

pub struct VDomElement {
    pub name: &'static str,
    pub attr: HashMap<&'static str, String>,
    pub child: Vec<VDomNode>,
    pub on_click: Option<Rc<dyn Fn()>>,
    pub on_input: Option<Rc<dyn Fn(String)>>,
    pub on_mouse_enter: Option<Rc<dyn Fn()>>,
    pub on_mouse_leave: Option<Rc<dyn Fn()>>,
    pub css: Option<Css>,
}

impl VDomElement {
    pub fn new(name: &'static str, child_list: Vec<NodeAttr>) -> VDomElement {
        let mut result = VDomElement {
            name,
            attr: HashMap::new(),
            child: Vec::new(),
            on_click: None,
            on_input: None,
            on_mouse_enter: None,
            on_mouse_leave: None,
            css: None,
        };

        for child in child_list {
            match child {
                NodeAttr::Css { css } => {
                    result.css = Some(css);
                },
                NodeAttr::OnClick { event } => {
                    result.on_click = Some(event);
                },
                NodeAttr::OnInput { event } => {
                    result.on_input = Some(event);
                },
                NodeAttr::OnMouseEnter { event } => {
                    result.on_mouse_enter = Some(event);
                },
                NodeAttr::OnMouseLeave { event } => {
                    result.on_mouse_leave = Some(event);
                },
                NodeAttr::Attr { name , value} => {
                    result.attr.insert(name, value);
                },
                NodeAttr::Node { node } => {
                    result.child.push(node);
                }
            }
        }

        result
    }

    pub fn new_with_v_dom(name: &'static str, child_list: Vec<VDomNode>) -> VDomElement {
        VDomElement {
            name,
            attr: HashMap::new(),
            child: child_list,
            on_click: None,
            on_input: None,
            on_mouse_enter: None,
            on_mouse_leave: None,
            css: None,
        }
    }
}

impl PartialEq for VDomElement {
    fn eq(&self, _other: &VDomElement) -> bool {
        false                                       //Always not-eq
    }
}
