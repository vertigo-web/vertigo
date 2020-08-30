use std::collections::HashMap;
use std::rc::Rc;
use crate::vdom::{
    models::{
        RealDomId::RealDomId,
        RealDomChild::RealDomChild,
    },
    DomDriver::{
        DomDriver::DomDriver,
    },
};

pub struct RealDomNode {
    domDriver: DomDriver,
    pub idDom: RealDomId,
    pub name: String,
    attr: HashMap<String, String>,
    pub child: RealDomChild,
}

impl RealDomNode {
    pub fn new(driver: DomDriver, name: String) -> RealDomNode {
        let nodeId = RealDomId::new();

        driver.createNode(nodeId.clone(), &name);

        let domChild = RealDomChild::newWithParent(driver.clone(), nodeId.clone());

        let node = RealDomNode {
            domDriver: driver,
            idDom: nodeId,
            name,
            attr: HashMap::new(),
            child: domChild,
        };

        node
    }

    fn updateAttrOne(&mut self, name: &String, value: &String) {
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

    pub fn updateAttr(&mut self, attr: &HashMap<String, String>) {
        self.attr.retain(|key, _value| {
            attr.contains_key(key)
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
