use std::collections::{
    HashMap,
    VecDeque,
};
use std::hash::Hash;

use crate::{
    computed::{
        Client,
    },
    driver::EventCallback
};

use crate::{
    virtualdom::{
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
            RealDomId::RealDomId,
        }
    },
    css_manager::css_manager::CssManager,
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

enum NodePairs<'a> {
    Component {
        real: &'a RealDomComponent,
        new: &'a VDomComponent
    },
    Node {
        real: &'a RealDomNode,
        new: &'a VDomNode,
    },
    Text {
        real: &'a RealDomText,
        new: &'a VDomText,
    }
}

fn get_pair_for_update<'a>(real: &'a RealDom, new: &'a VDom) -> Option<NodePairs<'a>> {
    match real {
        RealDom::Component { node } => {
            if let VDom::Component { node: vnode } = new {
                if node.id == vnode.id {
                    return Some(NodePairs::Component {
                        real: node,
                        new: vnode
                    });
                }
            }
        },
        RealDom::Node { node } => {
            if let VDom::Node { node : vnode} = new {
                if node.name() == vnode.name {
                    return Some(NodePairs::Node {
                        real: node,
                        new: vnode,
                    });
                }
            }
        },
        RealDom::Text { node } => {
            if let VDom::Text { node: vnode } = new {
                return Some(NodePairs::Text {
                    real: node,
                    new: vnode
                });
            }
        }
    }

    None
}

fn updateNodeChildUpdatedWithOrder(cssManager: &CssManager, target: &Vec<RealDom>, newVersion: &Vec<VDom>) -> bool {
    if target.len() != newVersion.len() {
        return false;
    }

    let max_index = target.len();

    let mut for_update: Vec<NodePairs> = Vec::new();

    for index in 0..max_index {
        let real = &target[index];
        let new = &newVersion[index];

        if let Some(pair) = get_pair_for_update(real, new) {
            for_update.push(pair);
        } else {
            return false;
        }
    }

    for item in for_update {
        match item {
            NodePairs::Component { real: _real, new: _new } => {
            },
            NodePairs::Node { real, new } => {
                updateNodeAttr(&cssManager, real, new);
                updateNodeChild(cssManager, real, new);
            },
            NodePairs::Text { real, new } => {
                real.update(&new.value);
            },
        }
    }

    true
}

fn updateNodeChild(cssManager: &CssManager, target: &RealDomNode, newVersion: &VDomNode) {

    let mut realChild = target.extract_child();

    let update_order_ok = updateNodeChildUpdatedWithOrder(cssManager, &mut realChild, &newVersion.child);
    if update_order_ok {
        target.put_child(realChild);
        return;
    }

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

    for item in realChild {
        match item {
            RealDom::Node { node }=> {
                realNode.insert(node.name(), node);
            },
            RealDom::Text { node } => {
                let id = node.get_value();
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
                let domChild = realText.getOrCreate(cssManager, target,id, node);
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


fn updateNodeAttr(css_manager: &CssManager, real_node: &RealDomNode, node: &VDomNode) {
    let css = &node.css;
    let class_name = match css {
        Some (css) => Some(css_manager.get_class_name(css)),
        None => None,
    };

    real_node.updateAttr(&node.attr, class_name);
    real_node.setEvent(EventCallback::OnClick { callback: node.onClick.clone() });
    real_node.setEvent(EventCallback::OnInput { callback: node.onInput.clone() });
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
