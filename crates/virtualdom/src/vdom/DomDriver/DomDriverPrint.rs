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
        log::info!("createNode {} - {}", id, name);
    }

    fn createText(&self, id: RealDomId, value: &String) {
        log::info!("createText {} {}", id, value);
    }

    fn createComment(&self, id: RealDomId, value: &String) {
        log::info!("createComment {} {}", id, value);
    }

    fn setAttr(&self, id: RealDomId, key: &String, value: &String) {
        log::info!("setAttr {} {} {}", id, key, value);
    }
    
    fn removeAttr(&self, id: RealDomId, name: &String) {
        log::info!("removeAttr {} {}", id, name);
    }

    fn remove(&self, id: RealDomId) {
        log::info!("remove {}", id);
    }

    fn removeAllChild(&self, id: RealDomId) {
        log::info!("removeAllChild {}", id);
    }

    fn insertAsFirstChild(&self, parent: RealDomId, child: RealDomId) {
        log::info!("insertAsFirstChild {} {}", parent, child);
    }

    fn insertBefore(&self, refId: RealDomId, child: RealDomId) {
        log::info!("insertBefore {} {}", refId, child);
    }

    fn insertAfter(&self, refId: RealDomId, child: RealDomId) {
        log::info!("insertAfter {} {}", refId, child);
    }

    fn addChild(&self, parent: RealDomId, child: RealDomId) {
        log::info!("addChild {} {}", parent, child);
    }
}

