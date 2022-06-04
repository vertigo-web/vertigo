use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};
use crate::{
    driver_module::driver_browser::{Driver},
    driver_module::driver_browser::{EventCallback},
    virtualdom::models::{
        dom::DomNode,
        dom_id::DomId,
    }, get_driver, Css, Client, Computed, struct_mut::VecMut, KeyDownEvent, DropFileEvent,
};

use crate::struct_mut::{
    ValueMut,
    VecDequeMut,
    HashMapMut,
};

fn merge_attr(attr: &HashMap<&'static str, String>, class_name: Option<String>) -> HashMap<&'static str, String> {
    let mut attr = attr.clone();

    if let Some(class_name) = class_name {
        let attr_class = attr.get("class");

        let value_to_set: String = match attr_class {
            Some(attr_class) => format!("{class_name} {attr_class}"),
            None => class_name,
        };

        attr.insert("class", value_to_set);
    }

    attr
}

pub enum AttrValue<T: Into<String> + Clone + PartialEq + 'static> {
    String(T),
    Computed(Computed<T>)
}

impl From<&'static str> for AttrValue<&'static str> {
    fn from(value: &'static str) -> Self {
        AttrValue::String(value)
    }
}

impl From<String> for AttrValue<String> {
    fn from(value: String) -> Self {
        AttrValue::String(value)
    }
}

impl From<Computed<String>> for AttrValue<String> {
    fn from(value: Computed<String>) -> Self {
        AttrValue::Computed(value)
    }
}

pub enum CssValue {
    Css(Css),
    Computed(Computed<Css>),
}

impl From<Css> for CssValue {
    fn from(value: Css) -> Self {
        CssValue::Css(value)
    }
}

impl From<Computed<Css>> for CssValue {
    fn from(value: Computed<Css>) -> Self {
        CssValue::Computed(value)
    }
}

pub struct DomNodeInner {
    dom_driver: Driver,
    pub id_dom: DomId,
    pub name: ValueMut<&'static str>,                   //TODO - Delete when the virtual dom is deleted
    attr: HashMapMut<&'static str, String>,             //TODO - Delete when the virtual dom is deleted
    child_node: VecDequeMut<DomNode>,
    subscriptions: VecMut<Client>,
}

impl Drop for DomNodeInner {
    fn drop(&mut self) {
        self.dom_driver.remove_node(self.id_dom);
    }
}

pub struct DomElement {
    inner: Rc<DomNodeInner>,
}

impl DomElement {
    pub fn new(name: &'static str) -> DomElement {
        let node_id = DomId::default();

        let driver = get_driver();

        driver.create_node(node_id, name);

        DomElement {
            inner: Rc::new(
                DomNodeInner {
                    dom_driver: driver,
                    id_dom: node_id,
                    name: ValueMut::new(name),
                    attr: HashMapMut::new(),
                    child_node: VecDequeMut::new(),
                    subscriptions: VecMut::new(),
                }
            ),
        }
    }

    pub fn create_with_id(id: DomId) -> DomElement {
        let driver = get_driver();

        DomElement {
            inner: Rc::new(
                DomNodeInner {
                    dom_driver: driver,
                    id_dom: id,
                    name: ValueMut::new("div"),
                    attr: HashMapMut::new(),
                    child_node: VecDequeMut::new(),
                    subscriptions: VecMut::new(),
                }
            ),
        }
    }

    fn subscribe<T: Clone + PartialEq + 'static>(&self, value: Computed<T>, call: impl Fn(T) + 'static) {
        let client = value.subscribe(call);
        self.inner.subscriptions.push(client);
    }

    pub fn css(self, css: CssValue) -> Self {
        match css {
            CssValue::Css(css) => {
                let class_name = get_driver().get_class_name(&css);
                self.inner.dom_driver.set_attr(self.inner.id_dom, "class", &class_name);             //TODO - Change to &str when the virtual dom is deleted        
            },
            CssValue::Computed(css) => {
                let id_dom = self.inner.id_dom;
                let driver = self.inner.dom_driver.clone();
        
                self.subscribe(css, move |css| {
                    let class_name = driver.get_class_name(&css);
                    driver.set_attr(id_dom, "class", &class_name);                                  //TODO - Change to &str when the virtual dom is deleted
                });
            }
        }
        self
    }

    pub fn update_attr(&self, attr: &HashMap<&'static str, String>, class_name: Option<String>) {              //TODO - Delete when the virtual dom is deleted
        let attr = merge_attr(attr, class_name);

        self.inner.attr.retain({
            let driver = self.inner.dom_driver.clone();
            let id_dom = self.inner.id_dom;
            let attr = &attr;

            move |key, _value| {
                let is_retain = attr.contains_key(key);
                if !is_retain {
                    driver.remove_attr(id_dom, key)
                }

                is_retain
            }
        });

        for (name, value) in attr.iter() {
            let need_update = self.inner.attr.insert_and_check(name, value.to_string());

            if need_update {
                self.inner.dom_driver.set_attr(self.inner.id_dom, name, value);
            }
        }
    }

    pub fn attr<T: Into<String> + Clone + PartialEq + 'static>(self, name: &'static str, value: AttrValue<T>) -> Self {
        match value {
            AttrValue::String(value) => {
                let id_dom = self.inner.id_dom;
                let value: String = value.into();
                get_driver().set_attr(id_dom, name, &value);        
            },
            AttrValue::Computed(computed) => {
                let id_dom = self.inner.id_dom;
                let driver = get_driver();
        
                self.subscribe(computed, move |value| {
                    let value: String = value.into();
                    driver.set_attr(id_dom, name, &value);
                });
        
            }
        };

        self
    }

    pub fn get_attr(&self, name: &'static str) -> Option<String> {              //TODO - Delete when the virtual dom is deleted
        self.inner.attr.get(&name)
    }

    pub fn set_event(&self, callback: EventCallback) {                          //TODO - Delete when the virtual dom is deleted
        self.inner.dom_driver.set_event(self.inner.id_dom, callback);
    }

    pub fn id_dom(&self) -> DomId {
        self.inner.id_dom
    }

    pub fn name(&self) -> &'static str {
        self.inner.name.get()
    }

    pub fn update_name(&self, name: &'static str) {                                     //TODO - Delete when the virtual dom is deleted
        let should_update = self.inner.name.set_and_check(name);

        if should_update {
            self.inner.dom_driver.rename_node(self.inner.id_dom, name);       //TODO - Delete when the virtual dom is deleted
        }
    }

    pub fn extract_child(&self) -> VecDeque<DomNode> {                                  //TODO - Delete when the virtual dom is deleted
        self.inner.child_node.take()
    }

    pub fn put_child(&self, child: VecDeque<DomNode>) {                                 //TODO - Delete when the virtual dom is deleted
        self.inner.child_node.replace(child);
    }

    pub fn insert_before(&self, new_child: DomId, prev_node: Option<DomId>) {                           //TODO - Delete when the virtual dom is deleted
        self.inner.dom_driver.insert_before(self.inner.id_dom, new_child, prev_node);
    }

    pub fn add_child(&self, child_node: impl Into<DomNode>) {
        let parent_id = self.inner.id_dom;
        let child_node = child_node.into().run_on_mount(parent_id);
        let child_id = child_node.id_dom();
        self.inner.dom_driver.insert_before(self.inner.id_dom, child_id, None);
        self.inner.child_node.push(child_node);
    }

    pub fn child(self, child_node: impl Into<DomNode>) -> Self {
        self.add_child(child_node);
        self
    }

    pub fn on_click(self, on_click: impl Fn() + 'static) -> Self {
        let callback = EventCallback::OnClick { callback: Some(Rc::new(on_click)) };
        self.inner.dom_driver.set_event(self.inner.id_dom, callback);
        self
    }

    pub fn on_mouse_enter(self, on_mouse_enter: impl Fn() + 'static) -> Self {
        let callback = EventCallback::OnMouseEnter { callback: Some(Rc::new(on_mouse_enter)) };
        self.inner.dom_driver.set_event(self.inner.id_dom, callback);
        self
    }

    pub fn on_mouse_leave(self, on_mouse_leave: impl Fn() + 'static) -> Self {
        let callback = EventCallback::OnMouseLeave { callback: Some(Rc::new(on_mouse_leave)) };
        self.inner.dom_driver.set_event(self.inner.id_dom, callback);
        self
    }

    pub fn on_input(self, on_input: impl Fn(String) + 'static) -> Self {
        let callback = EventCallback::OnInput { callback: Some(Rc::new(on_input)) };
        self.inner.dom_driver.set_event(self.inner.id_dom, callback);
        self
    }

    pub fn on_key_down(self, on_key_down: impl Fn(KeyDownEvent) -> bool + 'static) -> Self {
        let callback = EventCallback::OnKeyDown { callback: Some(Rc::new(on_key_down)) };
        self.inner.dom_driver.set_event(self.inner.id_dom, callback);
        self
    }

    pub fn on_dropfile(self, on_dropfile: impl Fn(DropFileEvent) + 'static) -> Self {
        let callback = EventCallback::OnDropFile { callback: Some(Rc::new(on_dropfile)) };
        self.inner.dom_driver.set_event(self.inner.id_dom, callback);
        self
    }

    pub fn hook_key_down(self, on_hook_key_down: impl Fn(KeyDownEvent) -> bool + 'static) -> Self {
        let callback = EventCallback::HookKeyDown { callback: Some(Rc::new(on_hook_key_down)) };
        self.inner.dom_driver.set_event(self.inner.id_dom, callback);
        self
    }

}

impl Clone for DomElement {                             //TODO - to be deleted after removal of VDom
    fn clone(&self) -> Self {
        DomElement {
            inner: self.inner.clone(),
        }
    }
}
