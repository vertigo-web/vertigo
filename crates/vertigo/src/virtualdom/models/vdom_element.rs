use std::{
    cmp::PartialEq,
    collections::{BTreeMap, HashMap},
    fmt,
    rc::Rc,
};
use crate::virtualdom::models::{
    css::Css,
    node_attr::NodeAttr,
    vdom_node::VDomNode,
};

use super::vdom_refs::NodeRefs;

//https://docs.rs/web-sys/0.3.50/web_sys/struct.KeyboardEvent.html

/// Structure passed as a parameter to callback on on_key_down event.
#[derive(Debug, Clone)]
pub struct KeyDownEvent {
    pub key: String,
    pub code: String,
    pub alt_key: bool,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub meta_key: bool,
}

impl std::fmt::Display for KeyDownEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KeyDownEvent={}", self.key)
    }
}

/// Virtual DOM node that represents a DOM element, a basic building block.
///
/// Usually returned from a render function:
///
/// ```rust,no_run
/// use vertigo::{VDomElement, html};
///
/// fn my_render() -> VDomElement {
///     html! {
///         <div>
///             <p>"Hello!"</p>
///         </div>
///     }
/// }
/// ```
pub struct VDomElement {
    pub name: &'static str,
    pub attr: HashMap<&'static str, String>,
    pub children: Vec<VDomNode>,
    pub dom_ref: Option<&'static str>,
    pub dom_apply: Option<Rc<dyn Fn(&NodeRefs)>>,
    pub on_click: Option<Rc<dyn Fn()>>,
    pub on_input: Option<Rc<dyn Fn(String)>>,
    pub on_mouse_enter: Option<Rc<dyn Fn()>>,
    pub on_mouse_leave: Option<Rc<dyn Fn()>>,
    pub on_key_down: Option<Rc<dyn Fn(KeyDownEvent) -> bool>>,
    pub hook_key_down: Option<Rc<dyn Fn(KeyDownEvent) -> bool>>,
    pub css: Option<Css>,
}

impl VDomElement {
    pub fn new(name: &'static str, attr_list: Vec<NodeAttr>, children: Vec<VDomNode>) -> Self {
        let mut result = Self {
            name,
            attr: HashMap::new(),
            children,
            dom_ref: None,
            dom_apply: None,
            on_click: None,
            on_input: None,
            on_mouse_enter: None,
            on_mouse_leave: None,
            on_key_down: None,
            hook_key_down: None,
            css: None,
        };

        for child in attr_list {
            match child {
                NodeAttr::Css { css } => {
                    result.css = Some(css);
                }
                NodeAttr::OnClick { event } => {
                    result.on_click = Some(event);
                }
                NodeAttr::OnInput { event } => {
                    result.on_input = Some(event);
                }
                NodeAttr::OnMouseEnter { event } => {
                    result.on_mouse_enter = Some(event);
                }
                NodeAttr::OnMouseLeave { event } => {
                    result.on_mouse_leave = Some(event);
                }
                NodeAttr::OnKeyDown { event } => {
                    result.on_key_down = Some(event);
                }
                NodeAttr::HookKeyDown { event } => {
                    result.hook_key_down = Some(event);
                }
                NodeAttr::Attr { name, value } => {
                    result.attr.insert(name, value);
                }
                NodeAttr::DomRef { name } => {
                    result.dom_ref = Some(name);
                }
                NodeAttr::DomApply { apply } => {
                    result.dom_apply = Some(apply);
                }
            }
        }

        result
    }

    pub fn build(name: &'static str) -> Self {
        Self {
            name,
            attr: HashMap::new(),
            children: Vec::new(),
            dom_ref: None,
            dom_apply: None,
            on_click: None,
            on_input: None,
            on_mouse_enter: None,
            on_mouse_leave: None,
            on_key_down: None,
            hook_key_down: None,
            css: None,
        }
    }

    #[must_use]
    pub fn attr<T: Into<String>>(mut self, attr: &'static str, value: T) -> Self {
        self.attr.insert(attr, value.into());
        self
    }

    #[must_use]
    pub fn css(mut self, css: Css) -> Self {
        self.css = Some(css);
        self
    }

    #[must_use]
    pub fn children(mut self, children: Vec<VDomNode>) -> Self {
        self.children = children;
        self
    }
}

impl PartialEq for VDomElement {
    fn eq(&self, _other: &Self) -> bool {
        false // Always not-eq
    }
}

impl fmt::Debug for VDomElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VDomElement")
            .field("name", &self.name)
            // Convert HashMap to BTreeMap to have attributes always in the same order
            .field("attr", &self.attr.iter().collect::<BTreeMap<_,_>>())
            .field("children", &self.children)
            .field("on_click", &self.on_click.as_ref().map(|f| f.as_ref() as *const dyn Fn()))
            .field("on_input", &self.on_input.as_ref().map(|f| f.as_ref() as *const dyn Fn(String)))
            .field("on_mouse_enter", &self.on_mouse_enter.as_ref().map(|f| f.as_ref() as *const dyn Fn()))
            .field("on_mouse_leave", &self.on_mouse_leave.as_ref().map(|f| f.as_ref() as *const dyn Fn()))
            .field("on_key_down", &self.on_key_down.as_ref().map(|f| f.as_ref() as *const dyn Fn(KeyDownEvent) -> bool))
            .field("css", &self.css)
            .finish()
    }
}
