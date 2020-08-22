use std::rc::Rc;
use crate::lib::BoxRefCell::BoxRefCell;
use crate::vdom::{
    DomDriver::{
        DomDriver::DomDriver,
    },
    models::{
        RealDom::RealDom,
        RealDomNodeId::RealDomNodeId,
        RealDomComment::RealDomComment,
    },
};

struct RealDomChildInner {
    pub first: RealDom,
    pub child: Vec<RealDom>, 
}

impl RealDomChildInner {
    pub fn new(first: RealDom) -> RealDomChildInner {
        RealDomChildInner {
            first: first,
            child: Vec::new(),
        }
    }

    pub fn newWithParent(driver: DomDriver, parent: RealDomNodeId) -> RealDomChildInner {
        let nodeComment = RealDomComment::new(driver.clone(), "".into());
        //driver.removeAllChild(RealDomNodeId::root());
        driver.insertAsFirstChild(parent, nodeComment.idDom.clone());
        let nodeList = RealDomChildInner::new(RealDom::Comment {
            node: nodeComment
        });

        nodeList
    }
}

pub struct RealDomChild {
    inner: Rc<BoxRefCell<RealDomChildInner>>,
}

impl RealDomChild {
    // pub fn new(first: RealDom) -> RealDomChild {

    //     RealDomChild {
    //         inner: Rc::new(BoxRefCell::new(
    //             RealDomChildInner::new(first)
    //         ))
    //     }
    // }

    pub fn newWithParent(driver: DomDriver, parent: RealDomNodeId) -> RealDomChild {
        RealDomChild {
            inner: Rc::new(BoxRefCell::new(
                RealDomChildInner::newWithParent(driver, parent)
            ))
        }
    }
}

impl Clone for RealDomChild {
    fn clone(&self) -> Self {
        RealDomChild {
            inner: self.inner.clone()
        }
    }
}