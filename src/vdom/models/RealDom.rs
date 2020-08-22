use crate::lib::{
    Client::Client,
};

use crate::vdom::{
    models::{
        Handler::Handler,
        Component::{
            ComponentId,
        },
        RealDomNode::RealDomNode,
        RealDomText::RealDomText,
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
        domDriver: DomDriver,
        id: ComponentId,                        //do porównywania
        subscription: Client,                   //Subskrybcją, , wstawia do handler
        handler: Handler,
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
