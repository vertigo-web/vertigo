use std::rc::Rc;
use crate::{NodeRefs, NodeRefsItem};

pub struct RefsContext {
    apply: Vec<Rc<dyn Fn(&NodeRefs)>>,
    node_refs: NodeRefs,
}

impl Default for RefsContext {
    fn default() -> Self {
        RefsContext {
            apply: Vec::new(),
            node_refs: NodeRefs::new(),
        }    
    }
}

impl RefsContext {
    pub fn run(self) {
        let RefsContext { apply, node_refs} = self;

        for apply_fun in apply {
            apply_fun(&node_refs);
        }
    }

    pub fn set_ref(&mut self, name: &'static str, item: NodeRefsItem) {
        self.node_refs.set(name, item);
    }

    pub fn add_apply(&mut self, dom_apply: &Rc<dyn Fn(&NodeRefs)>) {
        self.apply.push(dom_apply.clone());
    }
}
