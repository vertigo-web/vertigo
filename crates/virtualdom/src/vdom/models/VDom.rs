use crate::vdom::models::{
    VDomComponent::VDomComponent,
    VDomNode::VDomNode,
    VDomText::VDomText,
};
use std::rc::Rc;
use std::collections::HashMap;

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

impl VDom {
    pub fn node<T: Into<String>>(name: T) -> VDom {
        VDom::Node {
            node: VDomNode {
                name: name.into(),
                attr: HashMap::new(),
                child: Vec::new(),
                onClick: None,
            }
        }
    }

    pub fn text<T: Into<String>>(value: T) -> VDom {
        VDom::Text {
            node: VDomText {
                value: value.into()
            }
        }
    }

    pub fn attr<T: Into<String>, K: Into<String>>(mut self, name: T, value: K) -> Self {
        match &mut self {
            VDom::Node { node } => {
                node.attr.insert(name.into(), value.into());
            },
            _ => {
                panic!("Atrybut mozna dodac tylko do Node");
            }
        };

        self
    }

    pub fn child(mut self, child: VDom) -> Self {
        match &mut self {
            VDom::Node { node } => {
                node.child.push(child)
            },
            _ => {
                panic!("Nowy child mozna dodac tylko do Node");
            }
        };

        self
    }

    pub fn onClick<F: Fn() + 'static>(mut self, callback: F) -> Self {
        match &mut self {
            VDom::Node { node } => {
                node.onClick = Some(Rc::new(callback));
            },
            _ => {
                panic!("Nowy onClick mozna dodac tylko do Node");
            }
        };
    
        self
    }
}
