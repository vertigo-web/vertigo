use crate::lib::BoxRefCell::BoxRefCell;
use crate::vdom::{
    models::{
        RealDom::RealDom,
        RealDomNodeId::RealDomNodeId,
    },
    DomDriver::DomDriver::DomDriver,
};

#[derive(Clone)]
pub enum HandlerTarget {
    Parent(RealDomNodeId),          //oznacza ze zaczynamy wstawiac elementy jako pierwsze dziecko
    Prev(RealDomNodeId),            //pokazuje poprzedni element przed zakresem
}

impl HandlerTarget {
    pub fn root() -> HandlerTarget {
        HandlerTarget::Parent(RealDomNodeId::root())
    }
}

pub struct Handler {
    pub domDriver: DomDriver,
    pub targetToRender: BoxRefCell<HandlerTarget>,
    pub child: BoxRefCell<Vec<RealDom>>,
}

impl Handler {
    pub fn new(domDriver: DomDriver, target: HandlerTarget) -> Handler {
        Handler {
            domDriver: domDriver,
            targetToRender: BoxRefCell::new(target),
            child: BoxRefCell::new(Vec::new())
        }
    }
}