use std::rc::Rc;

use crate::virtualdom::models::css::Css;

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

pub fn attr<K: Into<String>>(name: &'static str, value: K) -> NodeAttr {
    NodeAttr::Attr {
        name,
        value: value.into()
    }
}
