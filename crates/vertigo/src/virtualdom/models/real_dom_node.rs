use std::collections::{
    HashMap,
    VecDeque,
};
use std::rc::Rc;
use crate::{driver::{DomDriver, EventCallback}, virtualdom::{
        models::{
            real_dom::RealDom,
            real_dom_id::RealDomId,
            real_dom_text::RealDomText,
        },
    }};
use crate::utils::BoxRefCell;


fn mergeAttr(attr: &HashMap<&'static str, String>, className: Option<String>) -> HashMap<&'static str, String> {
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
    attr: HashMap<&'static str, String>,
    pub child: VecDeque<RealDom>,
}

impl RealDomNodeInner {
    pub fn new(driver: DomDriver, name: &'static str) -> RealDomNodeInner {
        let nodeId = RealDomId::default();

        driver.create_node(nodeId.clone(), name);

        RealDomNodeInner {
            domDriver: driver,
            idDom: nodeId,
            name,
            attr: HashMap::new(),
            child: VecDeque::new(),
        }
    }

    pub fn createWithId(driver: DomDriver, id: RealDomId) -> RealDomNodeInner {
        RealDomNodeInner {
            domDriver: driver,
            idDom: id,
            name: "div",
            attr: HashMap::new(),
            child: VecDeque::new(),
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
            self.attr.insert(name, value.to_string());
       }
    }

    pub fn updateAttr(&mut self, attr: &HashMap<&'static str, String>, className: Option<String>) {
        let attr = mergeAttr(attr, className);

        let mut to_delate: Vec<&str> = Vec::new();

        for (key, _) in self.attr.iter() {
            if !attr.contains_key(*key) {
                to_delate.push(*key);
            }
        }

        for key_to_delete in to_delate.into_iter() {
            self.domDriver.remove_attr(self.idDom.clone(), key_to_delete)
        }

        self.attr.retain(|key, _value| {
            let key: &str = *key;

            attr.contains_key(key)
        });

        for (key, value) in attr.iter() {
            self.updateAttrOne(key, value);
        }
    }

    pub fn setEvent(&mut self, callback: EventCallback) {
        self.domDriver.set_event(self.idDom.clone(), callback);
    }

    pub fn extract_child(&mut self) -> VecDeque<RealDom> {
        std::mem::replace(&mut self.child, VecDeque::new())
    }

    pub fn put_child(&mut self, child: VecDeque<RealDom>) -> VecDeque<RealDom> {
        std::mem::replace(&mut self.child, child)
    }

    pub fn insert_before(&mut self, new_child: RealDom, prev_node: Option<RealDomId>) {
        self.domDriver.insert_before(self.idDom.clone(), new_child.id(), prev_node);
        self.child.push_front(new_child);
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

    pub fn updateAttr(&self, attr: &HashMap<&'static str, String>, className: Option<String>) {
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

    pub fn extract_child(&self) -> VecDeque<RealDom> {
        self.inner.change(
            (),
            |state, ()| {
                state.extract_child()
        })
    }

    pub fn put_child(&self, child: VecDeque<RealDom>) {
        self.inner.change(child, |state, child| {
            state.put_child(child);
        })
    }

    pub fn insert_before(&self, new_child: RealDom, prev_node: Option<RealDomId>) {
        self.inner.change(
            (new_child, prev_node),
            |state, (new_child, prev_node)| {
                state.insert_before(new_child, prev_node)
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