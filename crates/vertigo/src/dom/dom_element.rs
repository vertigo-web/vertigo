use std::rc::Rc;
use crate::{
    driver_module::{driver::{Driver}, api::DomAccess},
    dom::{
        dom_node::DomNode,
        dom_id::DomId,
    }, get_driver, Css, Client, Computed, struct_mut::VecMut, ApiImport,
};

use super::{types::{KeyDownEvent, DropFileEvent}, dom_node::{DomNodeFragment}};
use crate::struct_mut::VecDequeMut;

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

#[derive(Clone)]
pub struct DomElementRef {
    api: Rc<ApiImport>,
    id: DomId,
}

impl DomElementRef {
    pub fn new(api: Rc<ApiImport>, id: DomId) -> DomElementRef {
        DomElementRef {
            api,
            id,
        }
    }

    pub fn dom_access(&self) -> DomAccess {
        self.api.dom_access().element(self.id.to_u64())
    }
}

pub struct DomElement {
    driver: Driver,
    id_dom: DomId,
    child_node: VecDequeMut<DomNode>,
    subscriptions: VecMut<Client>,
}

impl DomElement {
    pub fn new(name: &'static str) -> DomElement {
        let node_id = DomId::default();

        let driver = get_driver();

        driver.create_node(node_id, name);

        DomElement {
            driver,
            id_dom: node_id,
            child_node: VecDequeMut::new(),
            subscriptions: VecMut::new(),
        }
    }

    pub fn get_ref(&self) -> DomElementRef {
        DomElementRef::new(self.driver.inner.api.clone(), self.id_dom)
    }

    pub fn create_with_id(id: DomId) -> DomElement {
        let driver = get_driver();

        DomElement {
            driver,
            id_dom: id,
            child_node: VecDequeMut::new(),
            subscriptions: VecMut::new(),
        }
    }

    fn subscribe<T: Clone + PartialEq + 'static>(&self, value: Computed<T>, call: impl Fn(T) + 'static) {
        let client = value.subscribe(call);
        self.subscriptions.push(client);
    }

    pub fn css(self, css: CssValue) -> Self {
        match css {
            CssValue::Css(css) => {
                let class_name = get_driver().get_class_name(&css);
                self.driver.set_attr(self.id_dom, "class", &class_name);             //TODO - Change to &str when the virtual dom is deleted        
            },
            CssValue::Computed(css) => {
                let id_dom = self.id_dom;
                let driver = self.driver.clone();
        
                self.subscribe(css, move |css| {
                    let class_name = driver.get_class_name(&css);
                    driver.set_attr(id_dom, "class", &class_name);                                  //TODO - Change to &str when the virtual dom is deleted
                });
            }
        }
        self
    }

    pub fn attr<T: Into<String> + Clone + PartialEq + 'static>(self, name: &'static str, value: AttrValue<T>) -> Self {
        match value {
            AttrValue::String(value) => {
                let id_dom = self.id_dom;
                let value: String = value.into();
                get_driver().set_attr(id_dom, name, &value);        
            },
            AttrValue::Computed(computed) => {
                let id_dom = self.id_dom;
                let driver = get_driver();
        
                self.subscribe(computed, move |value| {
                    let value: String = value.into();
                    driver.set_attr(id_dom, name, &value);
                });
        
            }
        };

        self
    }

    pub fn id_dom(&self) -> DomId {
        self.id_dom
    }

    pub fn add_child(&self, child_node: impl Into<DomNodeFragment>) {
        let parent_id = self.id_dom;
        let child_node = child_node.into().convert_to_node(parent_id);
        let child_id = child_node.id_dom();
        self.driver.insert_before(self.id_dom, child_id, None);
        self.child_node.push(child_node);
    }

    pub fn child(self, child_node: impl Into<DomNodeFragment>) -> Self {
        self.add_child(child_node);
        self
    }

    pub fn on_click(self, on_click: impl Fn() + 'static) -> Self {
        self.driver.inner.dom.data.change(self.id_dom, |element| {
            element.on_click = Some(Rc::new(on_click));
        });

        self
    }

    pub fn on_mouse_enter(self, on_mouse_enter: impl Fn() + 'static) -> Self {
        self.driver.inner.dom.data.change(self.id_dom, |element| {
            element.on_mouse_enter = Some(Rc::new(on_mouse_enter));
        });
        self
    }

    pub fn on_mouse_leave(self, on_mouse_leave: impl Fn() + 'static) -> Self {
        self.driver.inner.dom.data.change(self.id_dom, |element| {
            element.on_mouse_leave = Some(Rc::new(on_mouse_leave));
        });
        self
    }

    pub fn on_input(self, on_input: impl Fn(String) + 'static) -> Self {
        self.driver.inner.dom.data.change(self.id_dom, |element| {
            element.on_input = Some(Rc::new(on_input));
        });
        self
    }

    pub fn on_key_down(self, on_key_down: impl Fn(KeyDownEvent) -> bool + 'static) -> Self {
        self.driver.inner.dom.data.change(self.id_dom, |element| {
            element.on_keydown = Some(Rc::new(on_key_down));
        });
        self
    }

    pub fn on_dropfile(self, on_dropfile: impl Fn(DropFileEvent) + 'static) -> Self {
        self.driver.inner.dom.data.change(self.id_dom, |element| {
            element.on_dropfile = Some(Rc::new(on_dropfile));
        });
        self
    }

    pub fn hook_key_down(self, on_hook_key_down: impl Fn(KeyDownEvent) -> bool + 'static) -> Self {
        self.driver.inner.dom.data.change(self.id_dom, |element| {
            element.hook_keydown = Some(Rc::new(on_hook_key_down));
        });
        self
    }

}

impl Drop for DomElement {
    fn drop(&mut self) {
        self.driver.remove_node(self.id_dom);
    }
}
