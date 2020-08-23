use std::rc::Rc;
use crate::vdom::models::{
    RealDomId::RealDomId,
};

pub trait DomDriverTrait {
    fn createNode(&self, id: RealDomId, name: &String);
    fn createText(&self, id: RealDomId, value: &String);
    fn createComment(&self, id: RealDomId, value: &String);
    fn setAttr(&self, id: RealDomId, key: &String, value: &String);
    fn removeAttr(&self, id: RealDomId, name: &String);
    fn remove(&self, id: RealDomId);
    fn removeAllChild(&self, id: RealDomId);
    fn insertAsFirstChild(&self, parent: RealDomId, child: RealDomId);
    fn insertBefore(&self, refId: RealDomId, child: RealDomId);
    fn insertAfter(&self, refId: RealDomId, child: RealDomId);
    fn addChild(&self, parent: RealDomId, child: RealDomId);
}


pub struct DomDriver {
    driver: Rc<dyn DomDriverTrait>,
}

impl DomDriver {
    pub fn new<T: DomDriverTrait + 'static>(driver: T) -> DomDriver {
        DomDriver {
            driver: Rc::new(driver)
        }
    }
}

impl Clone for DomDriver {
    fn clone(&self) -> DomDriver {
        DomDriver {
            driver: self.driver.clone()
        }
    }
}

impl DomDriver {
    pub fn createNode(&self, id: RealDomId, name: &String) {
        self.driver.createNode(id, name);
    }

    pub fn createText(&self, id: RealDomId, value: &String) {
        self.driver.createText(id, value);
    }

    pub fn createComment(&self, id: RealDomId, value: &String) {
        self.driver.createComment(id, value);
    }

    pub fn setAttr(&self, id: RealDomId, key: &String, value: &String) {
        self.driver.setAttr(id, key, value);
    }

    pub fn removeAttr(&self, id: RealDomId, name: &String) {
        self.driver.removeAttr(id, name);
    }

    pub fn remove(&self, id: RealDomId) {
        self.driver.remove(id);
    }

    pub fn removeAllChild(&self, id: RealDomId) {
        self.driver.removeAllChild(id);
    }

    pub fn insertAsFirstChild(&self, parent: RealDomId, child: RealDomId) {
        self.driver.insertAsFirstChild(parent, child);
    }

    pub fn insertBefore(&self, refId: RealDomId, child: RealDomId) {
        self.driver.insertBefore(refId, child);
    }

    pub fn insertAfter(&self, refId: RealDomId, child: RealDomId) {
        self.driver.insertAfter(refId, child);
    }

    pub fn addChild(&self, parent: RealDomId, child: RealDomId) {
        self.driver.addChild(parent, child);
    }
}
