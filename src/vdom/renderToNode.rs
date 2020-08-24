
use std::rc::Rc;
use std::collections::{
    HashMap,
    VecDeque,
};
use std::hash::Hash;

use crate::{
    lib::{
        Client::Client,
        Computed::Computed,
    },
    vdom::{
        DomDriver::DomDriver::DomDriver,
        models::{
            RealDom::RealDom,
            RealDomChild::RealDomChild,
            RealDomNode::RealDomNode,
            RealDomText::RealDomText,
            RealDomComponent::RealDomComponent,
            VDom::VDom,
            VDomNode::VDomNode,
            VDomComponent::VDomComponent,
            VDomComponentId::VDomComponentId,
            VDomText::VDomText,
        }
    }
};

struct CacheNode<K: Eq + Hash, RNode, VNode> {
    domDriver: DomDriver,
    createNew: fn(&DomDriver, &VNode) -> RNode,
    synchronize: fn (&mut RNode, &VNode),
    data: HashMap<K, VecDeque<RNode>>,
}

impl<K: Eq + Hash, RNode, VNode> CacheNode<K, RNode, VNode> {
    fn new(
        domDriver: DomDriver,
        createNew: fn(&DomDriver, &VNode) -> RNode,
        synchronize: fn (&mut RNode, &VNode)
    ) -> CacheNode<K, RNode, VNode> {
        CacheNode {
            domDriver,
            createNew,
            synchronize,
            data: HashMap::new()
        }
    }

    fn insert(&mut self, key: K, node: RNode) {
        let item = self.data.entry(key).or_insert_with(VecDeque::new);
        item.push_back(node);
    }

    fn getOrCreate(&mut self, key: K, vnode: &VNode) -> RNode {
        let item = self.data.entry(key).or_insert_with(VecDeque::new);

        let node = item.pop_front();

        let CacheNode { createNew, synchronize, .. } = self;

        match node {
            Some(mut node) => {
                synchronize(&mut node, &vnode);
                node
            },
            None => {
                createNew(&self.domDriver, &vnode)
            }
        }
    }
}

fn nodeCreateNew(driver: &DomDriver, node: &VDomNode) -> RealDomNode {
    let realNode = RealDomNode::new(driver.clone(), node.name.clone());
    todo!();
}

fn nodeSynchronize(real: &mut RealDomNode, node: &VDomNode) {
    todo!();
}

fn textCreateNew(driver: &DomDriver, node: &VDomText) -> RealDomText {
    RealDomText::new(driver.clone(), node.value.clone())
}

fn textSynchronize(real: &mut RealDomText, node: &VDomText) {
    real.value = node.value.clone();                                //niepotrzebne klonowanie prawdopodobnie
}

fn componentCreateNew(driver: &DomDriver, node: &VDomComponent) -> RealDomComponent {

    todo!();
    // let child = RealDomChild::newDetached(*driver);
    // let subscription = renderToNode(child.clone(), node.render.clone());
    
    
    
    //renderToNode(target: RealDomChild, computed: Computed<Rc<Vec<VDom>>>) -> Client { 

    // struct RealDomComponent {
    //     pub id: VDomComponentId,                    //do porównywania
    //     subscription: Client,                   //Subskrybcją, , wstawia do handler
    //     child: RealDomChild,
    // }

    // struct VDomComponent {
    //     pub id: VDomComponentId,
    //     render: Computed<Vec<VDom>>,
    // }
}

fn componentSynchronize(real: &mut RealDomComponent, node: &VDomComponent) {

    todo!();

    //nic nie trzeba synchronizować. Komponent sam się synchronizuje.
}

// fn applyNewViewNode(om_a: &RealDomNode, dom_b: &VDomNode) {
//     /*
//         zeby przystąpić do synchronizaczji dwóch elementów, typ węzła musi się zgadzać
//             RealDom::name musi mieć takie samo jak VDom:name
        
//         synchronizujemy atrybuty

//         potem trzeba będzie zsynchronizować eventy podpięte pod ten węzeł

//         potem przechodzimy do synchronizowania dzieci
//     */
//     todo!();
// }


fn applyNewViewChild(target: RealDomChild, newVersion: &Vec<VDom>) {

    let mut realNode: CacheNode<String, RealDomNode, VDomNode> = CacheNode::new(
        target.getDomDriver(),
        nodeCreateNew, 
        nodeSynchronize
    );
    let mut realText: CacheNode<String, RealDomText, VDomText> = CacheNode::new(                    //TODO - trzeci parametr HashMap<String, String>
        target.getDomDriver(),
        textCreateNew, 
        textSynchronize
    );
    let mut realComponent: CacheNode<VDomComponentId, RealDomComponent, VDomComponent> = CacheNode::new(
        target.getDomDriver(),
        componentCreateNew,
        componentSynchronize
    );

    let realChild = target.extract();

    for item in realChild {
        match item {
            RealDom::Node { node }=> {
                let id = node.name.clone();
                realNode.insert(id, node);
            },
            RealDom::Text { node } => {
                let id = node.value.clone();
                realText.insert(id, node);
            },
            RealDom::Component { node } => {
                let id = node.id.clone();
                realComponent.insert(id, node);
            }
        }
    }


    for item in newVersion.iter() {

        match item {
            VDom::Node { node } => {
                let id = node.name.clone();
                let domChild = realNode.getOrCreate(id, &node);

                applyNewViewChild(domChild.child.clone(), &node.child);

                target.append(RealDom::Node { node: domChild });
            },
            VDom::Text { node } => {
                let id = node.value.clone();
                let domChild = realText.getOrCreate(id, &node);
                
                target.append(RealDom::Text { node: domChild });
            },
            VDom::Component { node } => {
                let id = node.id.clone();
                let domChild = realComponent.getOrCreate(id, &node);

                target.append(RealDom::Component { node: domChild });
            }
        }
    }
}


pub fn renderToNode(target: RealDomChild, computed: Computed<Vec<VDom>>) -> Client { 
    let subscription: Client = computed.subscribe(move |newVersion| {
        applyNewViewChild(
            target.clone(), 
            newVersion
        );
    });

    subscription
}
