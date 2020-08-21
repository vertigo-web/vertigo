use crate::vdom::models::RealDom::{
    RealDomNodeId,
};

pub enum BrowserDomDriver {
    Print,
}


impl BrowserDomDriver {
    pub fn createNode(&self, id: RealDomNodeId, name: String) {
        println!("create node {} - {}", id, name);
    }
}

/*
createNode(id: RealNodeId, name: String)
createText(id: RealNodeId, value: String)

createAttr(id: RealNodeId, key: String, value: String)
removeAttr(id: RealNodeId, key: String)

addChild(idParent: RealNodeId, idPrev Option<RealNodeId>, idChild: RealNodeId)              gdyby nie ten Option, to mozna by sie pozbyc parenta
removeChild(idChild: RealNodeId)


węzeł o numerze 1, to jest root względem którego będziemy rysować
*/