use std::collections::{
    HashMap,
    VecDeque,
};
use std::hash::Hash;

use crate::{
    virtualdom::{
        models::{
            realdom_node::RealDomElement,
        }
    },
    css_manager::css_manager::CssManager,
};


pub struct CacheNode<K: Eq + Hash, RNode, VNode> {
    create_new: fn(&CssManager, &RealDomElement, &VNode) -> RNode,
    data: HashMap<K, VecDeque<RNode>>,
}

impl<K: Eq + Hash, RNode, VNode> CacheNode<K, RNode, VNode> {
    pub fn new(
        create_new: fn(&CssManager, &RealDomElement, &VNode) -> RNode,
    ) -> CacheNode<K, RNode, VNode> {
        CacheNode {
            create_new,
            data: HashMap::new()
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
            None => create_new(css_manager, target, &vnode)
        }
    }
}
