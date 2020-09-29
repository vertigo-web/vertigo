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

pub struct RealDomNode {
    domDriver: DomDriver,
    pub idDom: RealDomId,
    pub name: &'static str,
    attr: HashMap<&'static str, String>,
    pub child: RealDomChildList,
}

impl RealDomNode {
    pub fn new(driver: DomDriver, name: &'static str, refAfter: RealDomId) -> RealDomNode {
        let nodeId = RealDomId::new();

        driver.createNode(nodeId.clone(), name);
        driver.insertAfter(refAfter, nodeId.clone());

        let domChild = RealDomChildList::newWithParent(driver.clone(), nodeId.clone());

        let node = RealDomNode {
            domDriver: driver,
            idDom: nodeId,
            name,
            attr: HashMap::new(),
            child: domChild,
        };

        node
    }

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

impl Drop for RealDomNode {
    fn drop(&mut self) {
        self.domDriver.remove(self.idDom.clone());
    }
}
