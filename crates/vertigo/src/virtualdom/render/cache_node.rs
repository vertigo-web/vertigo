use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
};

use crate::{
    css::css_manager::CssManager,
    virtualdom::models::realdom_node::RealDomElement
};

pub struct CacheNode<K: Eq + Hash, RNode, VNode> {
    create_new: Box<dyn Fn(&CssManager, &RealDomElement, &VNode) -> RNode>,
    data: HashMap<K, VecDeque<RNode>>,
}

impl<K: Eq + Hash, RNode, VNode> CacheNode<K, RNode, VNode> {
    pub fn new(
        create_new: impl Fn(&CssManager, &RealDomElement, &VNode) -> RNode + 'static,
    ) -> CacheNode<K, RNode, VNode> {
        CacheNode {
            create_new: Box::new(create_new),
            data: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, node: RNode) {
        let item = self.data.entry(key).or_insert_with(VecDeque::new);
        item.push_back(node);
    }

    pub fn get_or_create(&mut self, css_manager: &CssManager, target: &RealDomElement, key: K, vnode: &VNode) -> RNode {
        let item = self.data.entry(key).or_insert_with(VecDeque::new);

        let node = item.pop_front();

        let CacheNode { create_new, .. } = self;

        match node {
            Some(node) => node,
            None => create_new(css_manager, target, vnode),
        }
    }
}
