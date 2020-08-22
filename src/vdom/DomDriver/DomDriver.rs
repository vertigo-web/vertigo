use std::rc::Rc;
use crate::vdom::models::RealDom::{
    RealDomNodeId,
};
pub trait DomDriverTrait {
    fn createNode(&self, id: RealDomNodeId, name: &String);
    fn createText(&self, id: RealDomNodeId, value: &String);
    fn setAttr(&self, id: RealDomNodeId, key: &String, value: &String);
    fn removeAttr(&self, id: RealDomNodeId, name: &String);
    fn addChild(&self, idParent: RealDomNodeId, idPrev: Option<RealDomNodeId>, idChild: RealDomNodeId);
    fn remove(&self, id: RealDomNodeId);
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

    pub fn setAttr(&self, id: RealDomNodeId, key: &String, value: &String) {
        self.driver.setAttr(id, key, value);
    }

    pub fn removeAttr(&self, id: RealDomNodeId, name: &String) {
        self.driver.removeAttr(id, name);
    }

    pub fn addChild(&self, idParent: RealDomNodeId, idPrev: Option<RealDomNodeId>, idChild: RealDomNodeId) {
        self.driver.addChild(idParent, idPrev, idChild);
    }

    pub fn remove(&self, id: RealDomNodeId) {
        self.driver.remove(id);
    }
}
