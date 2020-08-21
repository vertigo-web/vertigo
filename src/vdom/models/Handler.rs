use crate::lib::BoxRefCell::BoxRefCell;
use crate::vdom::models::{
    RealNodeId::RealNodeId,
    RealDom::RealDom,
};

#[derive(Clone)]
pub enum HandlerTarget {
    Parent(RealNodeId),          //oznacza ze zaczynamy wstawiac elementy jako pierwsze dziecko
    Prev(RealNodeId),            //pokazuje poprzedni element przed zakresem
}

impl HandlerTarget {
    pub fn root() -> HandlerTarget {
        HandlerTarget::Parent(RealNodeId::root())
    }
}

pub struct Handler {
    pub targetToRender: BoxRefCell<HandlerTarget>,
    pub child: BoxRefCell<Vec<RealDom>>,
}

impl Handler {
    pub fn new(target: HandlerTarget) -> Handler {
        Handler {
            targetToRender: BoxRefCell::new(target),
            child: BoxRefCell::new(Vec::new())
        }
    }
}