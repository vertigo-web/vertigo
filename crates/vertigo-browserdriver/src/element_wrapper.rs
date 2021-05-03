use std::{
    rc::Rc,
};
use web_sys::{Element, Text};

pub enum ElementItem {
    Element {
        node: Element,
    },
    Text {
        text: Text
    },
}

impl ElementItem {
    pub fn from_node(node: Element) -> ElementItem {
        ElementItem::Element { node }
    }

    pub fn from_text(text: Text) -> ElementItem {
        ElementItem::Text { text }
    }
}

pub struct ElementWrapper {
    pub item: ElementItem,
    pub on_click: Option<Rc<dyn Fn()>>,
    pub on_input: Option<Rc<dyn Fn(String)>>,
    pub on_mouse_enter: Option<Rc<dyn Fn()>>,
    pub on_mouse_leave: Option<Rc<dyn Fn()>>,
}

impl ElementWrapper {
    pub fn from_node(node: Element) -> ElementWrapper {
        ElementWrapper {
            item: ElementItem::from_node(node),
            on_click: None,
            on_input: None,
            on_mouse_enter: None,
            on_mouse_leave: None,
        }
    }

    pub fn from_text(text: Text) -> ElementWrapper {
        ElementWrapper {
            item: ElementItem::from_text(text),
            on_click: None,
            on_input: None,
            on_mouse_enter: None,
            on_mouse_leave: None,
        }
    }
}
