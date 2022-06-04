use std::rc::Rc;
use crate::DropFileEvent;
use crate::virtualdom::models::css::Css;

use super::vdom_element::KeyDownEvent;

/// Virtual DOM node attribute.
pub enum NodeAttr {
    Css { css: Css },
    OnClick { event: Rc<dyn Fn()> },
    OnInput { event: Rc<dyn Fn(String)> },
    OnMouseEnter { event: Rc<dyn Fn()> },
    OnMouseLeave { event: Rc<dyn Fn()> },
    OnKeyDown { event: Rc<dyn Fn(KeyDownEvent) -> bool> },
    HookKeyDown { event: Rc<dyn Fn(KeyDownEvent) -> bool> },
    OnDropFile { event: Rc<dyn Fn(DropFileEvent)> },
    Attr { name: &'static str, value: String },
}

pub fn css(css: Css) -> NodeAttr {
    NodeAttr::Css { css }
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

pub fn on_key_down<F: Fn(KeyDownEvent) -> bool + 'static>(callback: F) -> NodeAttr {
    NodeAttr::OnKeyDown {
        event: Rc::new(callback),
    }
}

pub fn hook_key_down<F: Fn(KeyDownEvent) -> bool + 'static>(callback: F) -> NodeAttr {
    NodeAttr::HookKeyDown {
        event: Rc::new(callback),
    }
}

pub fn on_dropfile<F: Fn(DropFileEvent) + 'static>(callback: F) -> NodeAttr {
    NodeAttr::OnDropFile {
        event: Rc::new(callback),
    }
}

pub fn attr<K: Into<String>>(name: &'static str, value: K) -> NodeAttr {
    NodeAttr::Attr {
        name,
        value: value.into(),
    }
}

