use std::collections::HashMap;
use std::rc::Rc;

use crate::vdom::models::{
    VDom::VDom,
};
use crate::vdom::models::{
    Css::{Css, CssFrames},
    NodeAttr::NodeAttr,
};

pub struct VDomNode {
    pub name: &'static str,
    pub attr: HashMap<&'static str, String>,
    pub child: Vec<VDom>,
    pub onClick: Option<Rc<dyn Fn()>>,
    pub onInput: Option<Rc<dyn Fn(String)>>,
    pub onMouseEnter: Option<Rc<dyn Fn()>>,
    pub onMouseLeave: Option<Rc<dyn Fn()>>,
    pub css: Option<Css>,
    pub cssFrames: Option<CssFrames>,
}

impl VDomNode {
    pub fn new(name: &'static str, childList: Vec<NodeAttr>) -> VDomNode {
        let mut result = VDomNode {
            name,
            attr: HashMap::new(),
            child: Vec::new(),
            onClick: None,
            onInput: None,
            onMouseEnter: None,
            onMouseLeave: None,
            css: None,
            cssFrames: None,
        };

        for child in childList {
            match child {
                NodeAttr::Css { css } => {
                    result.css = Some(css);
                },
                NodeAttr::CssFrames { frames } => {
                    result.cssFrames = Some(frames);
                },
                NodeAttr::OnClick { event } => {
                    result.onClick = Some(event);
                },
                NodeAttr::OnInput { event } => {
                    result.onInput = Some(event);
                },
                NodeAttr::onMouseEnter { event } => {
                    result.onMouseEnter = Some(event);
                },
                NodeAttr::onMouseLeave { event } => {
                    result.onMouseLeave = Some(event);
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

    pub fn newWithVDom(name: &'static str, childList: Vec<VDom>) -> VDomNode {
        VDomNode {
            name,
            attr: HashMap::new(),
            child: childList,
            onClick: None,
            onInput: None,
            onMouseEnter: None,
            onMouseLeave: None,
            css: None,
            cssFrames: None,
        }
    }
}
