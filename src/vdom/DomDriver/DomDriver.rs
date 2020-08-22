use std::rc::Rc;
use crate::vdom::models::{
    RealDomNodeId::RealDomNodeId,
};

pub trait DomDriverTrait {
    fn createNode(&self, id: RealDomNodeId, name: &String);
    fn createText(&self, id: RealDomNodeId, value: &String);
    fn createComment(&self, id: RealDomNodeId, value: &String);
    fn setAttr(&self, id: RealDomNodeId, key: &String, value: &String);
    fn removeAttr(&self, id: RealDomNodeId, name: &String);
    fn remove(&self, id: RealDomNodeId);
    fn removeAllChild(&self, id: RealDomNodeId);
    fn insertAsFirstChild(&self, parent: RealDomNodeId, child: RealDomNodeId);
    fn insertBefore(&self, refId: RealDomNodeId, child: RealDomNodeId);
    fn insertAfter(&self, refId: RealDomNodeId, child: RealDomNodeId);
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
    pub fn createNode(&self, id: RealDomNodeId, name: &String) {
        self.driver.createNode(id, name);
    }

    pub fn createText(&self, id: RealDomNodeId, value: &String) {
        self.driver.createText(id, value);
    }

    pub fn createComment(&self, id: RealDomNodeId, value: &String) {
        self.driver.createComment(id, value);
    }

    pub fn setAttr(&self, id: RealDomNodeId, key: &String, value: &String) {
        self.driver.setAttr(id, key, value);
    }

    pub fn removeAttr(&self, id: RealDomNodeId, name: &String) {
        self.driver.removeAttr(id, name);
    }

    pub fn remove(&self, id: RealDomNodeId) {
        self.driver.remove(id);
    }

    pub fn removeAllChild(&self, id: RealDomNodeId) {
        self.driver.removeAllChild(id);
    }

    pub fn insertAsFirstChild(&self, parent: RealDomNodeId, child: RealDomNodeId) {
        self.driver.insertAsFirstChild(parent, child);
    }

    pub fn insertBefore(&self, refId: RealDomNodeId, child: RealDomNodeId) {
        self.driver.insertBefore(refId, child);
    }

    pub fn insertAfter(&self, refId: RealDomNodeId, child: RealDomNodeId) {
        self.driver.insertAfter(refId, child);
    }
}
