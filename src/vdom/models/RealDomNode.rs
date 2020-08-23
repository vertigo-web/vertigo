use std::collections::HashMap;

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
    idDom: RealDomId,
    name: String,
    attr: HashMap<String, String>,
    child: RealDomChild,
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

    pub fn setAttr(&mut self, name: String, value: String) {
        let needUpdate = {
            let item = self.attr.get(&name);
            if let Some(item) = item {
                if *item == value {
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
             self.attr.insert(name, value);
       }
    }

    pub fn removeAttr(&mut self, name: String) {
        let needDelete = {
            self.attr.contains_key(&name)
        };

        if needDelete {
            self.attr.remove(&name);
            self.domDriver.removeAttr(self.idDom.clone(), &name);
        }
    }
}

impl Drop for RealDomNode {
    fn drop(&mut self) {
        self.domDriver.remove(self.idDom.clone());
    }
}
