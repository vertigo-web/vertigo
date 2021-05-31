use std::{
    rc::Rc,
};
use vertigo::{KeyDownEvent, NodeRefsItem, NodeRefsItemTrait};
use web_sys::{Element, Text};

pub struct ElementRef {
    node: Element,
}

impl ElementRef {
    pub fn new(node: Element) -> ElementRef {
        ElementRef {
            node,
        }
    }
}

impl NodeRefsItemTrait for ElementRef {
    fn get_bounding_client_rect(&self) -> (f64, f64, f64, f64) {
        let rect = self.node.get_bounding_client_rect();

        (
            rect.x(),
            rect.y(),
            rect.width(),
            rect.height()
        )
    }

    fn scroll_top(&self) -> i32 {
        self.node.scroll_top()
    }

    fn set_scroll_top(&self, value: i32) {
        self.node.set_scroll_top(value);
    }

    fn scroll_left(&self) -> i32 {
        self.node.scroll_left()
    }

    fn set_scroll_left(&self, value: i32) {
        self.node.set_scroll_left(value);
    }

    fn scroll_width(&self) -> i32 {
        self.node.scroll_width()
    }

    fn scroll_height(&self) -> i32 {
        self.node.scroll_height()
    }
}

#[derive(Clone)]
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

    pub fn to_ref(&self) -> Option<ElementRef> {
        if let ElementItem::Element { node } = self {
            return Some(ElementRef::new(node.clone()));
        }

        None
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

    pub fn to_ref(&self) -> Option<NodeRefsItem> {
        let element_ref = self.item.to_ref();
        
        if let Some(element_ref) = element_ref {
            return Some(NodeRefsItem::new(element_ref));
        }

        None
    }
}
