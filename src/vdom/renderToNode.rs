
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
    synchronize: fn (&DomDriver, &mut RNode, &VNode),
    data: HashMap<K, VecDeque<RNode>>,
}

impl<K: Eq + Hash, RNode, VNode> CacheNode<K, RNode, VNode> {
    fn new(
        domDriver: DomDriver,
        createNew: fn(&DomDriver, &VNode) -> RNode,
        synchronize: fn (&DomDriver, &mut RNode, &VNode)
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
                synchronize(&self.domDriver, &mut node, &vnode);
                node
            },
            None => {
                createNew(&self.domDriver, &vnode)
            }
        }
    }
}

fn nodeCreateNew(driver: &DomDriver, node: &VDomNode) -> RealDomNode {
    todo!();
}

fn nodeSynchronize(driver: &DomDriver, real: &mut RealDomNode, node: &VDomNode) {
    //let realNode = RealDomNode::new(driver.clone(), node.name.clone());
    todo!();
}

fn textCreateNew(driver: &DomDriver, node: &VDomText) -> RealDomText {
    todo!();
}

fn textSynchronize(driver: &DomDriver, real: &mut RealDomText, node: &VDomText) {
    todo!();
}

fn componentCreateNew(driver: &DomDriver, node: &VDomComponent) -> RealDomComponent {
    todo!();
}

fn componentSynchronize(driver: &DomDriver, real: &mut RealDomComponent, node: &VDomComponent) {
    todo!();
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


fn applyNewViewChild(target: RealDomChild, newVersion: Vec<VDom>) -> VecDeque<(RealDomChild, Vec<VDom>)> {

    let mut realNode: CacheNode<String, RealDomNode, VDomNode> = CacheNode::new(
        target.getDomDriver(),
        nodeCreateNew, 
        nodeSynchronize
    );
    let mut realText: CacheNode<String, RealDomText, VDomText> = CacheNode::new(
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


    let mut out = VecDeque::new();

    for item in newVersion.into_iter() {

        match item {
            VDom::Node { node } => {
                let id = node.name.clone();
                let domChild = realNode.getOrCreate(id, &node);

                out.push_back((
                    domChild.child.clone(),
                    node.child.clone()
                ));

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

    out
}


pub fn renderToNode(target: RealDomChild, computed: Computed<Rc<Vec<VDom>>>) -> Client { 
    let subscription: Client = computed.subscribe(move |newVersion| {
        let mut syncTodo: VecDeque<(RealDomChild, Vec<VDom>)> = applyNewViewChild(
            target.clone(), 
            (*(*newVersion)).clone()
        );

        loop {
            let first = syncTodo.pop_front();

            match first {
                Some((realList, virtualList)) => {
                    let newListTodo = applyNewViewChild(realList, virtualList);
                    syncTodo.extend(newListTodo);
                },
                None => {
                    return;
                }
            }
        }
    });

    subscription
}
