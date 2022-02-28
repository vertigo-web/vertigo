use std::rc::Rc;

use crate::virtualdom::models::css::Css;

use super::vdom_element::KeyDownEvent;
use super::vdom_refs::NodeRefs;

/// Virtual DOM node attribute.
pub enum NodeAttr {
    Css { css: Css },
    OnClick { event: Rc<dyn Fn()> },
    OnInput { event: Rc<dyn Fn(String)> },
    OnMouseEnter { event: Rc<dyn Fn()> },
    OnMouseLeave { event: Rc<dyn Fn()> },
    OnKeyDown { event: Rc<dyn Fn(KeyDownEvent) -> bool> },
    HookKeyDown { event: Rc<dyn Fn(KeyDownEvent) -> bool> },
    Attr { name: &'static str, value: String },
    DomRef { name: &'static str },
    DomApply { apply: Rc<dyn Fn(&NodeRefs)> },
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

pub fn attr<K: Into<String>>(name: &'static str, value: K) -> NodeAttr {
    NodeAttr::Attr {
        name,
        value: value.into(),
    }
}

pub fn dom_ref(name: &'static str) -> NodeAttr {
    NodeAttr::DomRef { name }
}

pub fn dom_apply<F: Fn(&NodeRefs) + 'static>(f: F) -> NodeAttr {
    NodeAttr::DomApply { apply: Rc::new(f) }
}
