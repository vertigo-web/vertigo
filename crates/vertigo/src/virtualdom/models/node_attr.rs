use std::rc::Rc;

use crate::virtualdom::models::css::Css;

use super::vdom_element::KeyDownEvent;

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
    OnMouseEnter {
        event: Rc<dyn Fn()>
    },
    OnMouseLeave {
        event: Rc<dyn Fn()>
    },
    OnKeyDown {
        event: Rc<dyn Fn(KeyDownEvent)>,
    },
    Attr {
        name: &'static str,
        value: String,
    },
}



pub fn css(css: Css) -> NodeAttr {
    NodeAttr::Css {
        css,
    }
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

pub fn on_key_down<F: Fn(KeyDownEvent) + 'static>(callback: F) -> NodeAttr {
    NodeAttr::OnKeyDown {
        event: Rc::new(callback),
    }
}

pub fn attr<K: Into<String>>(name: &'static str, value: K) -> NodeAttr {
    NodeAttr::Attr {
        name,
        value: value.into()
    }
}
