use std::rc::Rc;
use vertigo::KeyDownEvent;

pub struct DomElement {
    pub on_click: Option<Rc<dyn Fn()>>,
    pub on_input: Option<Rc<dyn Fn(String)>>,
    pub on_mouse_enter: Option<Rc<dyn Fn()>>,
    pub on_mouse_leave: Option<Rc<dyn Fn()>>,
    pub on_keydown: Option<Rc<dyn Fn(KeyDownEvent) -> bool>>,
    pub hook_keydown: Option<Rc<dyn Fn(KeyDownEvent) -> bool>>,
}

impl DomElement {
    pub(crate) fn new() -> DomElement {
        DomElement {
            on_click: None,
            on_input: None,
            on_mouse_enter: None,
            on_mouse_leave: None,
            on_keydown: None,
            hook_keydown: None,
        }
    }
}
