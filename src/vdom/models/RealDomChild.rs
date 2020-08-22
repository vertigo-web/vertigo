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

pub struct RealDomChild {
    pub first: BoxRefCell<RealDom>,
    pub child: BoxRefCell<Vec<RealDom>>,                //musi występować zawsze przynajmniej jeden element
}

impl RealDomChild {
    pub fn new(first: RealDom) -> RealDomChild {
        RealDomChild {
            first: BoxRefCell::new(first),
            child: BoxRefCell::new(Vec::new()),
        }
    }

    pub fn newWithParent(driver: DomDriver, parent: RealDomNodeId) -> RealDomChild {
        let nodeComment = RealDomComment::new(driver.clone(), "".into());
        //driver.removeAllChild(RealDomNodeId::root());
        driver.insertAsFirstChild(parent, nodeComment.idDom.clone());
        let nodeList = RealDomChild::new(RealDom::Comment {
            node: nodeComment
        });

        nodeList
    }
}