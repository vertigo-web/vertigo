use std::rc::Rc;
use crate::lib::BoxRefCell::BoxRefCell;
use crate::vdom::{
    DomDriver::{
        DomDriver::DomDriver,
    },
    models::{
        RealDom::RealDom,
        RealDomId::RealDomId,
        RealDomComment::RealDomComment,
    },
};

enum RealDomChildInner {
    Empty {
        comment: RealDomComment,
    },
    List {
        first: RealDom,
        child: Vec<RealDom>,
    }
}

impl RealDomChildInner {
    pub fn new(comment: RealDomComment) -> RealDomChildInner {
        RealDomChildInner::Empty {
            comment
        }
    }

    pub fn newWithParent(driver: DomDriver, parent: RealDomId) -> RealDomChildInner {
        let nodeComment = RealDomComment::new(driver.clone(), "".into());
        driver.addChild(parent, nodeComment.idDom.clone());
        let nodeList = RealDomChildInner::new(nodeComment);

        nodeList
    }
}

pub struct RealDomChild {
    inner: Rc<BoxRefCell<RealDomChildInner>>,
}

impl RealDomChild {
    pub fn newWithParent(driver: DomDriver, parent: RealDomId) -> RealDomChild {
        RealDomChild {
            inner: Rc::new(BoxRefCell::new(
                RealDomChildInner::newWithParent(driver, parent)
            ))
        }
    }

    pub fn extract(&self) -> Vec<RealDom> {
        todo!();
    }

    pub fn append(&self, child: RealDom) {
        todo!();
    }

    pub fn firstChildId(&self) -> RealDomId {
        todo!();
    }

    pub fn lastChildId(&self) -> RealDomId {
        todo!();
    }
}

impl Clone for RealDomChild {
    fn clone(&self) -> Self {
        RealDomChild {
            inner: self.inner.clone()
        }
    }
}