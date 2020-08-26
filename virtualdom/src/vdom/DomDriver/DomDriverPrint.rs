use crate::vdom::{
    models::{
        RealDomId::RealDomId,
    },
    DomDriver::DomDriver::DomDriverTrait,
};

pub struct DomDriverPrint {}

impl DomDriverPrint {
    pub fn new() -> DomDriverPrint {
        DomDriverPrint {}
    }
}

impl DomDriverTrait for DomDriverPrint {
    fn createNode(&self, id: RealDomId, name: &String) {
        println!("createNode {} - {}", id, name);
    }

    fn createText(&self, id: RealDomId, value: &String) {
        println!("createText {} {}", id, value);
    }

    fn createComment(&self, id: RealDomId, value: &String) {
        println!("createComment {} {}", id, value);
    }

    fn setAttr(&self, id: RealDomId, key: &String, value: &String) {
        println!("setAttr {} {} {}", id, key, value);
    }
    
    fn removeAttr(&self, id: RealDomId, name: &String) {
        println!("removeAttr {} {}", id, name);
    }

    fn remove(&self, id: RealDomId) {
        println!("remove {}", id);
    }

    fn removeAllChild(&self, id: RealDomId) {
        println!("removeAllChild {}", id);
    }

    fn insertAsFirstChild(&self, parent: RealDomId, child: RealDomId) {
        println!("insertAsFirstChild {} {}", parent, child);
    }

    fn insertBefore(&self, refId: RealDomId, child: RealDomId) {
        println!("insertBefore {} {}", refId, child);
    }

    fn insertAfter(&self, refId: RealDomId, child: RealDomId) {
        println!("insertAfter {} {}", refId, child);
    }

    fn addChild(&self, parent: RealDomId, child: RealDomId) {
        println!("addChild {} {}", parent, child);
    }
}

