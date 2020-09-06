use std::collections::{
    HashMap,
    VecDeque,
};
use std::hash::Hash;

use crate::computed::{
    Client::Client,
    Computed::Computed,
};

use crate::{
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
            CssManager::CssManager,
        }
    }
};

struct CacheNode<K: Eq + Hash, RNode, VNode> {
    cssManager: CssManager,
    domDriver: DomDriver,
    createNew: fn(&CssManager, &DomDriver, &VNode) -> RNode,
    synchronize: fn (&CssManager, &mut RNode, &VNode),
    data: HashMap<K, VecDeque<RNode>>,
}

impl<K: Eq + Hash, RNode, VNode> CacheNode<K, RNode, VNode> {
    fn new(
        cssManager: CssManager,
        domDriver: DomDriver,
        createNew: fn(&CssManager, &DomDriver, &VNode) -> RNode,
        synchronize: fn (&CssManager, &mut RNode, &VNode)
    ) -> CacheNode<K, RNode, VNode> {
        CacheNode {
            cssManager,
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
                synchronize(&self.cssManager, &mut node, &vnode);
                node
            },
            None => {
                createNew(&self.cssManager, &self.domDriver, &vnode)
            }
        }
    }
}

fn nodeCreateNew(cssManager: &CssManager, driver: &DomDriver, node: &VDomNode) -> RealDomNode {
    let mut realNode = RealDomNode::new(driver.clone(), node.name.clone());

    let css = &node.css;
    let className = match css {
        Some (css) => Some(cssManager.getClassName(css)),
        None => None,
    };

    realNode.updateAttr(&node.attr, className);
    realNode.updateOnClick(node.onClick.clone());
    realNode
}

fn nodeSynchronize(cssManager: &CssManager, realNode: &mut RealDomNode, node: &VDomNode) {

    let css = &node.css;
    let className = match css {
        Some (css) => Some(cssManager.getClassName(css)),
        None => None,
    };

    realNode.updateAttr(&node.attr, className);
    realNode.updateOnClick(node.onClick.clone());
}

fn textCreateNew(cssManager: &CssManager, driver: &DomDriver, node: &VDomText) -> RealDomText {
    RealDomText::new(driver.clone(), node.value.clone())
}

fn textSynchronize(cssManager: &CssManager, real: &mut RealDomText, node: &VDomText) {
    real.update(&node.value);
}

fn componentCreateNew(cssManager: &CssManager, driver: &DomDriver, node: &VDomComponent) -> RealDomComponent {
    let child = RealDomChild::newDetached(driver.clone());
    let subscription = renderToNode(cssManager.clone(), child.clone(), node.render.clone());

    RealDomComponent {
        id: node.id.clone(),
        subscription,
        child
    }
}

fn componentSynchronize(_cssManager: &CssManager, _real: &mut RealDomComponent, _node: &VDomComponent) {
    //nic nie trzeba synchronizować. Komponent sam się synchronizuje.
}


fn applyNewViewChild(cssManager: CssManager, target: &RealDomChild, newVersion: &Vec<VDom>) {

    let mut realNode: CacheNode<&'static str, RealDomNode, VDomNode> = CacheNode::new(
        cssManager.clone(),
        target.getDomDriver(),
        nodeCreateNew, 
        nodeSynchronize
    );
    let mut realText: CacheNode<String, RealDomText, VDomText> = CacheNode::new(                    //TODO - trzeci parametr HashMap<String, String>
        cssManager.clone(),
        target.getDomDriver(),
        textCreateNew, 
        textSynchronize
    );
    let mut realComponent: CacheNode<VDomComponentId, RealDomComponent, VDomComponent> = CacheNode::new(
        cssManager.clone(),
        target.getDomDriver(),
        componentCreateNew,
        componentSynchronize
    );

    let realChild = target.extract();

    for item in realChild {
        match item {
            RealDom::Node { node }=> {
                realNode.insert(node.name, node);
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

                applyNewViewChild(cssManager.clone(), &domChild.child, &node.child);

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


pub fn renderToNode(cssManager: CssManager, target: RealDomChild, computed: Computed<Vec<VDom>>) -> Client { 
    let subscription: Client = computed.subscribe(move |newVersion| {
        applyNewViewChild(
            cssManager.clone(),
            &target, 
            newVersion
        );
    });

    subscription
}
