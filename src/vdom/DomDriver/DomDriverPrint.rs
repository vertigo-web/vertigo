use crate::vdom::{
    models::{
        RealDomNodeId::{
            RealDomNodeId,
        },
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
    fn createNode(&self, id: RealDomNodeId, name: &String) {
        log::info!("createNode {} - {}", id, name);
    }

    fn createText(&self, id: RealDomNodeId, value: &String) {
        log::info!("createText {} {}", id, value);
    }

    fn createComment(&self, id: RealDomNodeId, value: &String) {
        log::info!("createComment {} {}", id, value);
    }

    fn setAttr(&self, id: RealDomNodeId, key: &String, value: &String) {
        log::info!("setAttr {} {} {}", id, key, value);
    }
    
    fn removeAttr(&self, id: RealDomNodeId, name: &String) {
        log::info!("removeAttr {} {}", id, name);
    }

    fn remove(&self, id: RealDomNodeId) {
        log::info!("remove {}", id);
    }

    fn removeAllChild(&self, id: RealDomNodeId) {
        log::info!("removeAllChild {}", id);
    }

    fn insertAsFirstChild(&self, parent: RealDomNodeId, child: RealDomNodeId) {
        log::info!("insertAsFirstChild {} {}", parent, child);
    }

    fn insertBefore(&self, refId: RealDomNodeId, child: RealDomNodeId) {
        log::info!("insertBefore {} {}", refId, child);
    }

    fn insertAfter(&self, refId: RealDomNodeId, child: RealDomNodeId) {
        log::info!("insertAfter {} {}", refId, child);
    }
}

