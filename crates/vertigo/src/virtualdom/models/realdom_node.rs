use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use crate::{
    driver_module::driver_browser::{Driver},
    driver_module::driver_browser::{EventCallback},
    virtualdom::models::{
        realdom::RealDomNode,
        realdom_id::RealDomId,
        realdom_text::RealDomText,
        vdom_refs::NodeRefsItem,
    }
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

pub struct RealDomNodeInner {
    dom_driver: Driver,
    pub id_dom: RealDomId,
    pub name: ValueMut<&'static str>,
    attr: HashMapMut<&'static str, String>,
    pub child: VecDequeMut<RealDomNode>,
}

impl Drop for RealDomNodeInner {
    fn drop(&mut self) {
        self.dom_driver.remove_node(self.id_dom);
    }
}

pub struct RealDomElement {
    inner: Rc<RealDomNodeInner>,
}

impl RealDomElement {
    pub fn new(driver: Driver, name: &'static str) -> RealDomElement {
        let node_id = RealDomId::default();

        driver.create_node(node_id, name);

        RealDomElement {
            inner: Rc::new(
                RealDomNodeInner {
                    dom_driver: driver,
                    id_dom: node_id,
                    name: ValueMut::new(name),
                    attr: HashMapMut::new(),
                    child: VecDequeMut::new(),
                }
            ),
        }
    }

    pub fn create_with_id(driver: Driver, id: RealDomId) -> RealDomElement {
        RealDomElement {
            inner: Rc::new(
                RealDomNodeInner {
                    dom_driver: driver,
                    id_dom: id,
                    name: ValueMut::new("div"),
                    attr: HashMapMut::new(),
                    child: VecDequeMut::new(),
                }
            ),
        }
    }

    pub fn update_attr(&self, attr: &HashMap<&'static str, String>, class_name: Option<String>) {
        let attr = merge_attr(attr, class_name);

        self.inner.attr.retain({
            let driver = self.inner.dom_driver.clone();
            let id_dom = self.inner.id_dom;
            let attr = &attr;

            move |key, _value| {
                let key: &str = *key;

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

    pub fn get_attr(&self, name: &'static str) -> Option<String> {
        self.inner.attr.get(&name)
    }

    pub fn set_event(&self, callback: EventCallback) {
        self.inner.dom_driver.set_event(self.inner.id_dom, callback);
    }

    pub fn id_dom(&self) -> RealDomId {
        self.inner.id_dom
    }

    pub fn name(&self) -> &'static str {
        self.inner.name.get()
    }

    pub fn update_name(&self, name: &'static str) {
        let should_update = self.inner.name.set_and_check(name);

        if should_update {
            self.inner.dom_driver.rename_node(self.inner.id_dom, name);
        }
    }

    pub fn extract_child(&self) -> VecDeque<RealDomNode> {
        self.inner.child.take()
    }

    pub fn put_child(&self, child: VecDeque<RealDomNode>) {
        self.inner.child.replace(child);
    }

    pub fn insert_before(&self, new_child: RealDomId, prev_node: Option<RealDomId>) {
        self.inner.dom_driver.insert_before(self.inner.id_dom, new_child, prev_node);
    }

    fn dom_driver(&self) -> Driver {
        self.inner.dom_driver.clone()
    }

    pub fn create_node(&self, name: &'static str) -> RealDomElement {
        RealDomElement::new(self.dom_driver(), name)
    }

    pub fn create_text(&self, name: String) -> RealDomText {
        RealDomText::new(self.dom_driver(), name)
    }

    pub fn get_ref(&self) -> NodeRefsItem {
        let driver = self.inner.dom_driver.clone();
        let id = self.id_dom();

        NodeRefsItem::new(driver, id)
    }
}

impl Clone for RealDomElement {
    fn clone(&self) -> Self {
        RealDomElement {
            inner: self.inner.clone(),
        }
    }
}
