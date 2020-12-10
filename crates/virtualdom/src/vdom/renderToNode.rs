use std::collections::{
    HashMap,
    VecDeque,
};
use std::hash::Hash;

use crate::computed::{
    Client::Client,
};

use crate::{
    vdom::{
        models::{
            RealDom::RealDom,
            RealDomNode::RealDomNode,
            RealDomText::RealDomText,
            RealDomComponent::RealDomComponent,
            VDom::VDom,
            VDomNode::VDomNode,
            VDomComponent::VDomComponent,
            VDomComponentId::VDomComponentId,
            VDomText::VDomText,
            CssManager::CssManager,
            RealDomId::RealDomId,
        }
    }
};

struct CacheNode<K: Eq + Hash, RNode, VNode> {
    createNew: fn(&CssManager, &RealDomNode, &VNode) -> RNode,
    data: HashMap<K, VecDeque<RNode>>,
}

impl<K: Eq + Hash, RNode, VNode> CacheNode<K, RNode, VNode> {
    fn new(
        createNew: fn(&CssManager, &RealDomNode, &VNode) -> RNode,
    ) -> CacheNode<K, RNode, VNode> {
        CacheNode {
            createNew,
            data: HashMap::new()
        }
    }

    fn insert(&mut self, key: K, node: RNode) {
        let item = self.data.entry(key).or_insert_with(VecDeque::new);
        item.push_back(node);
    }

    fn getOrCreate(&mut self, cssManager: &CssManager, target: &RealDomNode, key: K, vnode: &VNode) -> RNode {
        let item = self.data.entry(key).or_insert_with(VecDeque::new);

        let node = item.pop_front();

        let CacheNode { createNew, .. } = self;

        match node {
            Some(node) => node,
            None => createNew(cssManager, target, &vnode)
        }
    }
}

/*
    abcdefgh

    bcdeafgh

    złap pierwszy nowy element,
    sprawdź czy kolejny element za nim jest tym którym powinien być

    wyznac duze fragmenty które się zgaedzają
    najmniejszy z elementów przenies na swoje miejsce
*/

fn updateNodeChild(cssManager: &CssManager, target: &RealDomNode, newVersion: &VDomNode) {

    let mut realNode: CacheNode<&'static str, RealDomNode, VDomNode> = CacheNode::new(
        |_cssManager: &CssManager, target: &RealDomNode, node: &VDomNode| -> RealDomNode {
            target.createNode(node.name)
        }, 
    );
    let mut realText: CacheNode<String, RealDomText, VDomText> = CacheNode::new(
        |_cssManager: &CssManager, target: &RealDomNode, node: &VDomText| -> RealDomText {
            target.createText(node.value.clone())
        },
    );
    let mut realComponent: CacheNode<VDomComponentId, RealDomComponent, VDomComponent> = CacheNode::new(
        |cssManager: &CssManager, target: &RealDomNode, node: &VDomComponent| -> RealDomComponent {

            let node_root = target.createNode("div");

            let node_root_for_id = node_root.clone();

            let subscription = renderToNode(cssManager.clone(), node_root, node.clone());

            RealDomComponent {
                id: node.id.clone(),
                subscription,
                node: node_root_for_id,
            }
        },
    );

    let realChild = target.extract_child();

    for item in realChild {
        match item {
            RealDom::Node { node }=> {
                realNode.insert(node.name(), node);
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


    let mut wsk: Option<RealDomId> = None;

    for item in newVersion.child.iter() {

        match item {
            VDom::Node { node } => {
                let id = node.name;
                let domChild = realNode.getOrCreate(cssManager, target, id, node);
                let newWsk = domChild.idDom();

                updateNodeAttr(&cssManager, &domChild, &node);
                updateNodeChild(cssManager, &domChild, &node);

                target.appendAfter(wsk, RealDom::Node { node: domChild });
                wsk = Some(newWsk);
            },
            VDom::Text { node } => {
                let id = node.value.clone();
                let mut domChild = realText.getOrCreate(cssManager, target,id, node);
                let newWsk = domChild.idDom.clone();

                domChild.update(&node.value);

                target.appendAfter(wsk, RealDom::Text { node: domChild });
                wsk = Some(newWsk);
            },
            VDom::Component { node } => {
                let id = node.id.clone();
                let domChild = realComponent.getOrCreate(cssManager, target,id, node);
                let newWsk = domChild.domId();

                target.appendAfter(wsk, RealDom::Component { node: domChild });
                wsk = Some(newWsk);
            }
        }
    }
}


fn updateNodeAttr(cssManager: &CssManager, realNode: &RealDomNode, node: &VDomNode) {
    let css = &node.css;
    let className = match css {
        Some (css) => Some(cssManager.getClassName(css)),
        None => None,
    };

    realNode.updateAttr(&node.attr, className);
    realNode.updateOnClick(node.onClick.clone());
}

fn updateNode(cssManager: &CssManager, target: &RealDomNode, newVersion: &VDomNode) {

    //updejt nazwy taga ...
    //TODO !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

    //updejt atrybutów
    updateNodeAttr(&cssManager, target, &newVersion);

    //odpal updejt dzieci
    updateNodeChild(cssManager, target, &newVersion);
}

pub fn renderToNode(cssManager: CssManager, target: RealDomNode, component: VDomComponent) -> Client {
    let subscription: Client = component.view.subscribe(move |newVersion| {
        updateNode(
            &cssManager,
            &target, 
            newVersion
        );
    });

    subscription
}
