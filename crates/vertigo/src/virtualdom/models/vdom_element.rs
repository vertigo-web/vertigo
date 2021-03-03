use std::collections::HashMap;
use std::rc::Rc;
use std::cmp::PartialEq;
use std::fmt;

use crate::virtualdom::models::vdom_node::VDomNode;
use crate::virtualdom::models::{
    css::Css,
    node_attr::NodeAttr,
};

pub struct VDomElement {
    pub name: &'static str,
    pub attr: HashMap<&'static str, String>,
    pub children: Vec<VDomNode>,
    pub on_click: Option<Rc<dyn Fn()>>,
    pub on_input: Option<Rc<dyn Fn(String)>>,
    pub on_mouse_enter: Option<Rc<dyn Fn()>>,
    pub on_mouse_leave: Option<Rc<dyn Fn()>>,
    pub css: Option<Css>,
}

impl VDomElement {
    pub fn new(name: &'static str, attr_list: Vec<NodeAttr>, children: Vec<VDomNode>) -> Self {
        let mut result = VDomElement {
            name,
            attr: HashMap::new(),
            children,
            on_click: None,
            on_input: None,
            on_mouse_enter: None,
            on_mouse_leave: None,
            css: None,
        };

        for child in attr_list {
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
            }
        }

        result
    }
}

impl PartialEq for VDomElement {
    fn eq(&self, _other: &VDomElement) -> bool {
        false                                       //Always not-eq
    }
}

impl fmt::Debug for VDomElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VDomElement")
            .field("name", &self.name)
            .field("attr", &self.attr)
            .field("children", &self.children)
            .field("on_click", &self.on_click.as_ref().map(|f| f.as_ref() as *const dyn Fn()))
            .field("on_input", &self.on_input.as_ref().map(|f| f.as_ref() as *const dyn Fn(String)))
            .field("on_mouse_enter", &self.on_mouse_enter.as_ref().map(|f| f.as_ref() as *const dyn Fn()))
            .field("on_mouse_leave", &self.on_mouse_leave.as_ref().map(|f| f.as_ref() as *const dyn Fn()))
            .field("css", &self.css)
            .finish()
    }
}
