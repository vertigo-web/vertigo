
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
        models::{
            VDom::{VDom, VDomNode},
            RealDom::RealDom,
            RealDomChild::RealDomChild,
            RealDomNode::RealDomNode,
            RealDomText::RealDomText,
            RealDomComponent::RealDomComponent,
            VDomComponent::VDomComponent,
            VDomComponentId::VDomComponentId,
            VDomText::VDomText,
        }
    }
};

struct CacheNode<K: Eq + Hash, RNode, VNode> {
    createNew: fn(item: &VNode) -> RNode,
    synchronize: fn (&mut RNode, &VNode),
    data: HashMap<K, VecDeque<RNode>>,
}

impl<K: Eq + Hash, RNode, VNode> CacheNode<K, RNode, VNode> {
    fn new(createNew: fn(item: &VNode) -> RNode, synchronize: fn (&mut RNode, &VNode)) -> CacheNode<K, RNode, VNode> {
        CacheNode {
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
                createNew(&vnode)
            }
        }
    }
}

fn nodeCreateNew(node: &VDomNode) -> RealDomNode {
    todo!();
}

fn nodeSynchronize(real: &mut RealDomNode, node: &VDomNode) {
    todo!();
}

fn textCreateNew(node: &VDomText) -> RealDomText {
    todo!();
}

fn textSynchronize(real: &mut RealDomText, node: &VDomText) {
    todo!();
}

fn componentCreateNew(node: &VDomComponent) -> RealDomComponent {
    todo!();
}

fn componentSynchronize(real: &mut RealDomComponent, node: &VDomComponent) {
    todo!();
}

fn applyNewViewChild(target: RealDomChild, newVersion: Vec<VDom>) -> VecDeque<(RealDomChild, Vec<VDom>)> {

    let mut realNode: CacheNode<String, RealDomNode, VDomNode> = CacheNode::new(
        nodeCreateNew, 
        nodeSynchronize
    );
    let mut realText: CacheNode<String, RealDomText, VDomText> = CacheNode::new(
        textCreateNew, 
        textSynchronize
    );
    let mut realComponent: CacheNode<VDomComponentId, RealDomComponent, VDomComponent> = CacheNode::new(
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
                let item = realComponent.insert(id, node);
            }
        }
    }


    let mut out = VecDeque::new();

    for item in newVersion.into_iter() {

        match item {
            VDom::Node { node } => {
                let domChild = realNode.getOrCreate(node.name.clone(), &node);

                out.push_back((
                    domChild.child.clone(),
                    node.child.clone()
                ));

                target.append(RealDom::Node { node: domChild });
            },
            VDom::Text { node } => {
                //realText.getOrCreate(key, vnode)

                todo!();
            },
            VDom::Component { node } => {
                todo!();
            }
        }
    }

    out
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
