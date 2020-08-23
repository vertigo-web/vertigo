use crate::lib::{
    Client::Client,
};

use crate::vdom::{
    models::{
        RealDomNode::RealDomNode,
        RealDomText::RealDomText,
        RealDomChild::RealDomChild,
        VDomComponentId::VDomComponentId,
    },
    DomDriver::{
        DomDriver::DomDriver,
    },
};


pub enum RealDom {
    Node {
        node: RealDomNode,
    },
    Text {
        node: RealDomText,
    },
    Component {
        id: VDomComponentId,                        //do porównywania
        subscription: Client,                   //Subskrybcją, , wstawia do handler
        child: RealDomChild,
    }
}

impl RealDom {
    pub fn newNode(domDriver: DomDriver, name: String) -> RealDom {
        RealDom::Node {
            node: RealDomNode::new(domDriver, name)
        }
    }

    pub fn newText(domDriver: DomDriver, value: String) -> RealDom {
        RealDom::Text {
            node: RealDomText::new(domDriver, value)
        }
    }
}
