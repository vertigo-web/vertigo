use std::{
    rc::Rc,
};
use vertigo::{KeyDownEvent, NodeRefsItem, NodeRefsItemTrait, RealDomId};

use super::driver_browser_dom_js::DriverBrowserDomJs;

pub struct ElementRef {
    dom_js: Rc<DriverBrowserDomJs>,
    dom_id: RealDomId,
}

impl ElementRef {
    pub(crate) fn new(dom_js: Rc<DriverBrowserDomJs>, dom_id: RealDomId) -> ElementRef {
        ElementRef {
            dom_js,
            dom_id,
        }
    }
}

impl NodeRefsItemTrait for ElementRef {
    fn get_bounding_client_rect_x(&self) -> f64 {
        self.dom_js.get_bounding_client_rect_x(self.dom_id.to_u64())
    }

    fn get_bounding_client_rect_y(&self) -> f64 {
        self.dom_js.get_bounding_client_rect_y(self.dom_id.to_u64())
    }

    fn get_bounding_client_rect_width(&self) -> f64 {
        self.dom_js.get_bounding_client_rect_width(self.dom_id.to_u64())
    }

    fn get_bounding_client_rect_height(&self) -> f64 {
        self.dom_js.get_bounding_client_rect_height(self.dom_id.to_u64())
    }

    fn scroll_top(&self) -> i32 {
        self.dom_js.scroll_top(self.dom_id.to_u64())
    }

    fn set_scroll_top(&self, value: i32) {
        self.dom_js.set_scroll_top(self.dom_id.to_u64(), value);
    }

    fn scroll_left(&self) -> i32 {
        self.dom_js.scroll_left(self.dom_id.to_u64())
    }

    fn set_scroll_left(&self, value: i32) {
        self.dom_js.set_scroll_left(self.dom_id.to_u64(), value);
    }

    fn scroll_width(&self) -> i32 {
        self.dom_js.scroll_width(self.dom_id.to_u64())
    }

    fn scroll_height(&self) -> i32 {
        self.dom_js.scroll_height(self.dom_id.to_u64())
    }
}

pub struct DomElement {
    dom_js: Rc<DriverBrowserDomJs>,
    dom_id: RealDomId,
    pub on_click: Option<Rc<dyn Fn()>>,
    pub on_input: Option<Rc<dyn Fn(String)>>,
    pub on_mouse_enter: Option<Rc<dyn Fn()>>,
    pub on_mouse_leave: Option<Rc<dyn Fn()>>,
    pub on_keydown: Option<Rc<dyn Fn(KeyDownEvent) -> bool>>,
}

impl DomElement {
    pub(crate) fn new(dom_js: Rc<DriverBrowserDomJs>, dom_id: RealDomId, name: &str) -> DomElement {
        dom_js.create_node(dom_id.to_u64(), name);

        DomElement {
            dom_js,
            dom_id,
            on_click: None,
            on_input: None,
            on_mouse_enter: None,
            on_mouse_leave: None,
            on_keydown: None,
        }
    }

    pub fn set_attr(&self, attr: &str, value: &str) {
        self.dom_js.set_attribute(self.dom_id.to_u64(), attr, value);
    }

    pub fn remove_attr(&self, attr: &str) {
        self.dom_js.remove_attribute(self.dom_id.to_u64(), attr);
    }

    pub fn insert_before(&self, child: RealDomId, ref_id: Option<RealDomId>) {
        self.dom_js.insert_before(
            self.dom_id.to_u64(),
            child.to_u64(),
            ref_id.map(|id| id.to_u64())
        );
    }

    pub fn to_ref(&self) -> NodeRefsItem {
        NodeRefsItem::new(
            ElementRef::new(self.dom_js.clone(), self.dom_id.clone())
        )
    }
}

impl Drop for DomElement {
    fn drop(&mut self) {
        self.dom_js.remove_node(self.dom_id.to_u64());
    }
}

pub struct DomText {
    dom_js: Rc<DriverBrowserDomJs>,
    dom_id: RealDomId,
}

impl DomText {
    pub(crate) fn new(dom_js: Rc<DriverBrowserDomJs>, dom_id: RealDomId, value: &str) -> DomText {
        dom_js.create_text(dom_id.to_u64(), value);

        DomText {
            dom_js,
            dom_id
        }
    }

    pub fn update_text(&self, new_value: &str) {
        self.dom_js.update_text(self.dom_id.to_u64(), new_value);
    }

    pub fn insert_before(&self, child: RealDomId, ref_id: Option<RealDomId>) {
        self.dom_js.insert_before(
            self.dom_id.to_u64(),
            child.to_u64(),
            ref_id.map(|id| id.to_u64())
        );
    }
}

impl Drop for DomText {
    fn drop(&mut self) {
        self.dom_js.remove_text(self.dom_id.to_u64());
    }
}