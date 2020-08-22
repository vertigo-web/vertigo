use std::rc::Rc;
use crate::vdom::models::RealDom::{
    RealDomNodeId,
};
pub trait DomDriverTrait {
    fn createNode(&self, id: RealDomNodeId, name: String);
    fn createText(&self, id: RealDomNodeId, value: String);
    fn setAttr(&self, id: RealDomNodeId, key: String, value: String);
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
    fn createNode(&self, id: RealDomNodeId, name: String) {
        self.driver.createNode(id, name);
    }

    fn createText(&self, id: RealDomNodeId, value: String) {
        self.driver.createText(id, value);
    }

    fn setAttr(&self, id: RealDomNodeId, key: String, value: String) {
        self.driver.setAttr(id, key, value);
    }

    fn addChild(&self, idParent: RealDomNodeId, idPrev: Option<RealDomNodeId>, idChild: RealDomNodeId) {
        self.driver.addChild(idParent, idPrev, idChild);
    }

    fn remove(&self, id: RealDomNodeId) {
        self.driver.remove(id);
    }
}
