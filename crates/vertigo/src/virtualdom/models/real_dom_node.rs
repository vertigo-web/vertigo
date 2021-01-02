use alloc::{
    rc::Rc,
    collections::BTreeMap,
    string::String,
    vec::Vec,
    format,
};
use crate::{
    driver::{
        DomDriver,
        EventCallback
    },
    virtualdom::{
        models::{
            real_dom::RealDom,
            real_dom_id::RealDomId,
            real_dom_text::RealDomText,
        },
    },
};

use crate::utils::BoxRefCell;


fn mergeAttr(attr: &BTreeMap<&'static str, String>, className: Option<String>) -> BTreeMap<&'static str, String> {
    let mut attr = attr.clone();

    if let Some(className) = className {
        let attrClass = attr.get("class");

        let valueToSet: String = match attrClass {
            Some(attrClass) => format!("{} {}", className, attrClass),
            None => className
        };

        attr.insert("class", valueToSet);
    }

    attr
}

pub struct RealDomNodeInner {
    domDriver: DomDriver,
    pub idDom: RealDomId,
    pub name: &'static str,
    attr: BTreeMap<&'static str, String>,
    pub child: Vec<RealDom>,
}

impl RealDomNodeInner {
    pub fn new(driver: DomDriver, name: &'static str) -> RealDomNodeInner {
        let nodeId = RealDomId::default();

        driver.create_node(nodeId.clone(), name);

        RealDomNodeInner {
            domDriver: driver,
            idDom: nodeId,
            name,
            attr: BTreeMap::new(),
            child: Vec::new(),
        }
    }

    pub fn createWithId(driver: DomDriver, id: RealDomId) -> RealDomNodeInner {
        RealDomNodeInner {
            domDriver: driver,
            idDom: id,
            name: "div",
            attr: BTreeMap::new(),
            child: Vec::new(),
        }
    }

    fn updateAttrOne(&mut self, name: &'static str, value: &str) {
        let needUpdate = {
            let item = self.attr.get(name);
            if let Some(item) = item {
                *item != *value
            } else {
                true
            }
        };

        if needUpdate {
            self.domDriver.set_attr(self.idDom.clone(), &name, &value);
            self.attr.insert(name, value.into());
       }
    }

    pub fn updateAttr(&mut self, attr: &BTreeMap<&'static str, String>, className: Option<String>) {
        let attr = mergeAttr(attr, className);

        let mut to_delate: Vec<&str> = Vec::new();

        for (key, _) in self.attr.iter() {
            if !attr.contains_key(*key) {
                to_delate.push(*key);
            }
        }

        for key_to_delete in to_delate.into_iter() {
            self.domDriver.remove_attr(self.idDom.clone(), key_to_delete);
            self.attr.remove(key_to_delete);
        }

        for (key, value) in attr.iter() {
            self.updateAttrOne(key, value);
        }
    }

    pub fn setEvent(&mut self, callback: EventCallback) {
        self.domDriver.set_event(self.idDom.clone(), callback);
    }

    pub fn extract_child(&mut self) -> Vec<RealDom> {
        core::mem::replace(&mut self.child, Vec::new())
    }

    pub fn put_child(&mut self, child: Vec<RealDom>) -> Vec<RealDom> {
        core::mem::replace(&mut self.child, child)
    }

    pub fn appendAfter(&mut self, prevNode: Option<RealDomId>, newChild: RealDom) {
        match prevNode {
            Some(prevNode) => {
                self.domDriver.insert_after(prevNode, newChild.id());
            }
            None => {
                self.domDriver.add_child(self.idDom.clone(), newChild.id());
            }
        };

        self.child.push(newChild);
    }
}

impl Drop for RealDomNodeInner {
    fn drop(&mut self) {
        self.domDriver.remove(self.idDom.clone());
    }
}


pub struct RealDomNode {
    inner: Rc<BoxRefCell<RealDomNodeInner>>,
}

impl RealDomNode {
    pub fn new(driver: DomDriver, name: &'static str) -> RealDomNode {
        RealDomNode {
            inner: Rc::new(
                BoxRefCell::new(
                    RealDomNodeInner::new(driver, name)
                )
            )
        }
    }

    pub fn createWithId(driver: DomDriver, id: RealDomId) -> RealDomNode {
        RealDomNode {
            inner: Rc::new(
                BoxRefCell::new(
                    RealDomNodeInner::createWithId(driver, id)
                )
            )
        }
    }

    pub fn updateAttr(&self, attr: &BTreeMap<&'static str, String>, className: Option<String>) {
        self.inner.change(
            (attr, className),
            |state, (attr, className)| {
                state.updateAttr(attr, className)
        })
    }

    pub fn setEvent(&self, callback: EventCallback) {
        self.inner.change(
            callback,
            |state, callback| {
                state.setEvent(callback)
        })
    }

    pub fn idDom(&self) -> RealDomId {
        self.inner.get(
            |state| {
                state.idDom.clone()
        })
    }

    pub fn name(&self) -> &'static str {
        self.inner.get(
            |state| {
                state.name
        })
    }

    pub fn extract_child(&self) -> Vec<RealDom> {
        self.inner.change(
            (),
            |state, ()| {
                state.extract_child()
        })
    }

    pub fn put_child(&self, child: Vec<RealDom>) {
        self.inner.change(child, |state, child| {
            state.put_child(child);
        })
    }

    pub fn appendAfter(&self, prevNode: Option<RealDomId>, newChild: RealDom) {
        self.inner.change(
            (prevNode, newChild),
            |state, (prevNode, newChild)| {
                state.appendAfter(prevNode, newChild)
        })
    }

    fn domDriver(&self) -> DomDriver {
        self.inner.get(
            |state| {
                state.domDriver.clone()
        })
    }

    pub fn createNode(&self, name: &'static str) -> RealDomNode {
        RealDomNode::new(self.domDriver(), name)
    }

    pub fn createText(&self, name: String) -> RealDomText {
        RealDomText::new(self.domDriver(), name)
    }
}


impl Clone for RealDomNode {
    fn clone(&self) -> Self {
        RealDomNode {
            inner: self.inner.clone()
        }
    }
}