use std::rc::Rc;

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
    fn createNode(&self, id: RealDomId, name: &'static str) {
        log::info!("createNode {} - {}", id, name);
    }

    fn createText(&self, id: RealDomId, value: &String) {
        log::info!("createText {} {}", id, value);
    }

    fn createComment(&self, id: RealDomId, value: &String) {
        log::info!("createComment {} {}", id, value);
    }

    fn setAttr(&self, id: RealDomId, key: &'static str, value: &String) {
        log::info!("setAttr {} {} {}", id, key, value);
    }
    
    fn removeAttr(&self, id: RealDomId, name: &'static str) {
        log::info!("removeAttr {} {}", id, name);
    }

    fn remove(&self, id: RealDomId) {
        log::info!("remove {}", id);
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

    fn insertCss(&self, selector: String, value: String) {
        log::info!("insertCss {} {}", selector, value);
    }

    fn setOnClick(&self, node: RealDomId, onClick: Option<Rc<dyn Fn()>>) {
        let callback = match onClick {
            Some(_) => "Some(--callback--)",
            None => "None"
        };

        log::info!("setOnClick {} {}", node, callback);
    }
}

