use std::collections::HashMap;
use std::rc::Rc;
use crate::vdom::{
    models::{
        RealDomId::RealDomId,
        RealDomChildList::RealDomChildList,
    },
    DomDriver::{
        DomDriver::DomDriver,
    },
};
use crate::computed::BoxRefCell::BoxRefCell;

pub struct RealDomNodeInner {
    domDriver: DomDriver,
    pub idDom: RealDomId,
    pub name: &'static str,
    attr: HashMap<&'static str, String>,
    pub child: RealDomChildList,
}

impl RealDomNodeInner {
    pub fn new(driver: DomDriver, name: &'static str) -> RealDomNodeInner {
        let nodeId = RealDomId::new();

        driver.createNode(nodeId.clone(), name);

        let domChild = RealDomChildList::new(driver.clone(), nodeId.clone());

        let node = RealDomNodeInner {
            domDriver: driver,
            idDom: nodeId,
            name,
            attr: HashMap::new(),
            child: domChild,
        };

        node
    }

    pub fn createWithId(driver: DomDriver, id: RealDomId) -> RealDomNodeInner {
        let domChild = RealDomChildList::new(driver.clone(), id.clone());

        let node = RealDomNodeInner {
            domDriver: driver,
            idDom: id,
            name: "div",
            attr: HashMap::new(),
            child: domChild,
        };

        node
    }
                                                            //TODO - koniecznie pozbyć się tej funkcji
    fn updateAttrOne(&mut self, name: &'static str, value: &String) {
        let needUpdate = {
            let item = self.attr.get(name);
            if let Some(item) = item {
                if *item == *value {
                    false
                } else {
                    true
                }
            } else {
                true
            }
        };

        if needUpdate {
            self.domDriver.setAttr(self.idDom.clone(), &name, &value);
            self.attr.insert(name.clone(), value.clone());
       }
    }

    fn mergeAttr(&mut self, attr: &HashMap<&'static str, String>, className: Option<String>) -> HashMap<&'static str, String> {
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

    pub fn updateAttr(&mut self, attr: &HashMap<&'static str, String>, className: Option<String>) {
        let attr = self.mergeAttr(attr, className);

        self.attr.retain(|key, _value| {
            let key: &str = *key;

            let keyExist = attr.contains_key(key);
            keyExist
        });

        for (key, value) in attr.iter() {
            self.updateAttrOne(key, value);
        }
    }

    pub fn updateOnClick(&mut self, onClick: Option<Rc<dyn Fn()>>) {
        self.domDriver.setOnClick(self.idDom.clone(), onClick);
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

    pub fn updateOnClick(&self, onClick: Option<Rc<dyn Fn()>>) {
        self.inner.change(
            onClick,
            |state, onClick| {
                state.updateOnClick(onClick)
        })
    }

    pub fn child(&self) -> RealDomChildList {
        self.inner.get(
            |state| {
                state.child.clone()
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
}


impl Clone for RealDomNode {
    fn clone(&self) -> Self {
        RealDomNode {
            inner: self.inner.clone()
        }
    }
}