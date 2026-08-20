use std::{cell::RefCell, rc::Rc};

use super::graph::{ErasedNode, NodeId};

pub(crate) type ParentList = Vec<(NodeId, Rc<dyn ErasedNode>)>;

/// Tracking context passed into [`crate::Value::get`] / [`crate::Computed::get`].
pub struct Context {
    pub(crate) parents: Option<RefCell<ParentList>>,
}

impl Context {
    pub(crate) fn read() -> Self {
        Context { parents: None }
    }

    pub(crate) fn tracking() -> Self {
        Context {
            parents: Some(RefCell::new(Vec::new())),
        }
    }

    /// Record `id` as a parent of the node currently computing.
    ///
    /// A run of reads of the *same* node collapses into one entry. That is the shape a
    /// compute closure produces when it reads one value inside a loop, and it keeps the
    /// list proportional to the edges rather than to the reads - everything downstream
    /// (the strong ref kept per entry, building the id set in `Edges::replace`) is then
    /// paid once per parent.
    ///
    /// Only *consecutive* repeats collapse. Reads interleaved between several nodes still
    /// produce one entry each; `Edges::replace` folds those into a set anyway. A full
    /// deduplication here was measured slower on the ordinary many-distinct-parents path,
    /// which is why this stays a single comparison.
    pub(crate) fn track(&self, id: NodeId, slot: Rc<dyn ErasedNode>) {
        if let Some(parents) = &self.parents {
            let mut parents = parents.borrow_mut();
            if parents.last().map(|(last, _)| *last) == Some(id) {
                return;
            }
            parents.push((id, slot));
        }
    }

    pub(crate) fn take_parents(&self) -> ParentList {
        match &self.parents {
            Some(parents) => parents.take(),
            None => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct N;
    impl ErasedNode for N {
        fn refresh(&self) -> bool {
            false
        }
    }

    fn ids(parents: &ParentList) -> Vec<NodeId> {
        parents.iter().map(|(id, _)| *id).collect()
    }

    #[test]
    fn repeated_reads_of_one_node_collapse() {
        let ctx = Context::tracking();
        for _ in 0..100 {
            ctx.track(NodeId(1), Rc::new(N));
        }

        assert_eq!(ids(&ctx.take_parents()), vec![NodeId(1)]);
    }

    #[test]
    fn distinct_reads_are_all_kept() {
        let ctx = Context::tracking();
        ctx.track(NodeId(1), Rc::new(N));
        ctx.track(NodeId(2), Rc::new(N));
        ctx.track(NodeId(3), Rc::new(N));

        assert_eq!(
            ids(&ctx.take_parents()),
            vec![NodeId(1), NodeId(2), NodeId(3)]
        );
    }

    /// Interleaved repeats are left alone - `Edges::replace` folds them into a set.
    #[test]
    fn interleaved_repeats_are_left_for_the_edge_set() {
        let ctx = Context::tracking();
        ctx.track(NodeId(1), Rc::new(N));
        ctx.track(NodeId(2), Rc::new(N));
        ctx.track(NodeId(1), Rc::new(N));

        assert_eq!(
            ids(&ctx.take_parents()),
            vec![NodeId(1), NodeId(2), NodeId(1)]
        );
    }

    #[test]
    fn read_context_tracks_nothing() {
        let ctx = Context::read();
        ctx.track(NodeId(1), Rc::new(N));

        assert!(ctx.take_parents().is_empty());
    }
}
