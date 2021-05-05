use std::{
    rc::Rc,
};
use vertigo::KeyDownEvent;
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
    pub on_keydown: Option<Rc<dyn Fn(KeyDownEvent)>>,
}

impl ElementWrapper {
    fn from_element(element: ElementItem) -> ElementWrapper {
        ElementWrapper {
            item: element,
            on_click: None,
            on_input: None,
            on_mouse_enter: None,
            on_mouse_leave: None,
            on_keydown: None,
        }
    }

    pub fn from_node(node: Element) -> ElementWrapper {
        let item = ElementItem::from_node(node);
        ElementWrapper::from_element(item)
    }

    pub fn from_text(text: Text) -> ElementWrapper {
        let item = ElementItem::from_text(text);
        ElementWrapper::from_element(item)
    }
}
