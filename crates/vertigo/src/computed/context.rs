use std::collections::BTreeSet;

use super::{struct_mut::VecMut, GraphId};

pub enum Context {
    Computed { parent_ids: VecMut<GraphId> },
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

    pub(crate) fn add_parent(&self, parent_id: GraphId) {
        if let Context::Computed { parent_ids } = self {
            parent_ids.push(parent_id);
        }
    }

    pub(crate) fn get_parents(self) -> BTreeSet<GraphId> {
        if let Context::Computed { parent_ids } = self {
            parent_ids.into_inner().into_iter().collect::<BTreeSet<_>>()
        } else {
            BTreeSet::new()
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

    context.add_parent(id1);
    context.add_parent(id1);
    context.add_parent(id2);
    context.add_parent(id3);
    context.add_parent(id3);

    let list = context.get_parents();
    assert_eq!(list.len(), 3);
}
