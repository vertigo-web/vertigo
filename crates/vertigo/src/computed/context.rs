use std::any::Any;
use std::collections::BTreeSet;
use std::rc::Rc;

use super::{GraphId, struct_mut::VecMut};

pub enum Context {
    Computed {
        parent_ids: VecMut<(GraphId, Rc<dyn Any>)>,
    },
    Transaction,
}

impl Context {
    pub(crate) fn computed() -> Context {
        Context::Computed {
            parent_ids: VecMut::new(),
        }
    }

    pub(crate) fn transaction() -> Context {
        Context::Transaction
    }

    pub(crate) fn add_parent(&self, parent_id: GraphId, parent_rc: Rc<dyn Any>) {
        if let Context::Computed { parent_ids } = self {
            parent_ids.push((parent_id, parent_rc));
        }
    }

    pub(crate) fn get_parents(self) -> (BTreeSet<GraphId>, Vec<Rc<dyn Any>>) {
        if let Context::Computed { parent_ids } = self {
            parent_ids.into_inner().into_iter().unzip()
        } else {
            (BTreeSet::new(), Vec::new())
        }
    }

    pub(crate) fn is_transaction(&self) -> bool {
        match self {
            Context::Computed { parent_ids: _ } => false,
            Context::Transaction => true,
        }
    }
}

#[test]
fn test_context() {
    use crate::computed::graph_id::GraphIdKind;

    let context = Context::computed();

    let id1 = GraphId::new_for_test(GraphIdKind::Computed, 14);
    let id2 = GraphId::new_for_test(GraphIdKind::Computed, 15);
    let id3 = GraphId::new_for_test(GraphIdKind::Computed, 16);

    context.add_parent(id1, Rc::new(1));
    context.add_parent(id1, Rc::new(1));
    context.add_parent(id2, Rc::new(1));
    context.add_parent(id3, Rc::new(1));
    context.add_parent(id3, Rc::new(1));

    let (list, rcs) = context.get_parents();
    assert_eq!(list.len(), 3);
    assert_eq!(rcs.len(), 5);
}
