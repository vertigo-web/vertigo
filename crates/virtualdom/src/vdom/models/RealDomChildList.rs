use std::rc::Rc;
use crate::computed::BoxRefCell::BoxRefCell;
use crate::vdom::{
    DomDriver::{
        DomDriver::DomDriver,
    },
    models::{
        RealDom::RealDom,
        RealDomNode::RealDomNode,
        RealDomText::RealDomText,
        RealDomId::RealDomId,
    },
};


struct RealDomChildListInner {
    domDriver: DomDriver,
    parentId: RealDomId,
    child: Vec<RealDom>,
}

impl RealDomChildListInner {
    pub fn new(domDriver: DomDriver, parentId: RealDomId) -> RealDomChildListInner {
        RealDomChildListInner {
            domDriver,
            parentId,
            child: Vec::new(),
        }
    }

    pub fn extract(&mut self) -> Vec<RealDom> {
        let prev_child = std::mem::replace(&mut self.child, Vec::new());
        prev_child
    }

    pub fn appendAfter(&mut self, prevNode: Option<RealDomId>, newChild: RealDom) {
        match prevNode {
            Some(prevNode) => {
                self.domDriver.insertAfter(prevNode, newChild.id());
            }
            None => {
                self.domDriver.addChild(self.parentId.clone(), newChild.id());
            }
        };
        
        self.child.push(newChild);
    }

    fn createNode(&self, name: &'static str) -> RealDomNode {
        RealDomNode::new(self.domDriver.clone(), name)
    }

    fn createText(&self, name: String) -> RealDomText {
        RealDomText::new(self.domDriver.clone(), name)
    }
}

pub struct RealDomChildList {
    inner: Rc<BoxRefCell<RealDomChildListInner>>,
}

impl RealDomChildList {
    pub fn new(driver: DomDriver, parentId: RealDomId) -> RealDomChildList {
        RealDomChildList {
            inner: Rc::new(BoxRefCell::new(
                RealDomChildListInner::new(driver, parentId)
            ))
        }
    }

    pub fn extract(&self) -> Vec<RealDom> {
        self.inner.changeNoParams(|state| {
            state.extract()
        })
    }

    pub fn appendAfter(&self, prevNode: Option<RealDomId>, child: RealDom) {
        self.inner.change(
            (prevNode, child), 
            |state, (prevNode, child)| {
            state.appendAfter(prevNode, child)
        })
    }

    pub fn getDomDriver(&self) -> DomDriver {
        self.inner.get(|state| {
            state.domDriver.clone()
        })
    }

    pub fn createNode(&self, name: &'static str) -> RealDomNode {
        self.inner.getWithContext(name, |state, name| {
            state.createNode(name)
        })
    }

    pub fn createText(&self, name: String) -> RealDomText {
        self.inner.getWithContext(name, |state, name| {
            state.createText(name)
        })
    }
}

impl Clone for RealDomChildList {
    fn clone(&self) -> Self {
        RealDomChildList {
            inner: self.inner.clone()
        }
    }
}