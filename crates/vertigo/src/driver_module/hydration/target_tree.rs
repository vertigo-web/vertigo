use std::collections::{BTreeMap, HashMap};

use crate::{dev::command::DriverDomCommand, dom::dom_id::DomId, driver_module::StaticString};

#[derive(Debug, Clone, PartialEq)]
pub enum TargetKind {
    Element { name: StaticString },
    Text { value: String },
    Comment { value: String },
}

#[derive(Debug, Clone)]
pub struct TargetNode {
    pub kind: TargetKind,
    pub attrs: BTreeMap<StaticString, String>,
    pub children: Vec<DomId>,
}

/// The tree the application just built, reconstructed from its own command stream.
///
/// Rust does not know the contents of its own `DomNode` tree: `DomElement` does not store
/// attributes (`add_attr` writes them straight to the driver through a closure), and
/// `DomText` does not store text. The command stream is therefore the only description of
/// the tree, and this index is built from it.
///
/// It lives only for the duration of `DriverDom::flush_hydration` and is discarded afterward.
#[derive(Debug, Default)]
pub struct TargetTree {
    nodes: HashMap<DomId, TargetNode>,
    /// Parent of every inserted node. Kept separately so detaching costs one sibling list
    /// instead of scanning every node — the JavaScript equivalent searches the whole node map
    /// on every `InsertBefore`, which is quadratic on a large page.
    parent: HashMap<DomId, DomId>,
}

pub struct SplitBuffer {
    pub tree: TargetTree,
    /// Commands unrelated to node identity, sent unchanged.
    pub passthrough: Vec<DriverDomCommand>,
}

/// Splits a mount buffer into a tree index and commands that pass through unchanged.
pub fn split_buffer(commands: Vec<DriverDomCommand>) -> SplitBuffer {
    let mut tree = TargetTree::default();
    let mut passthrough = Vec::new();

    for command in commands {
        match command {
            DriverDomCommand::CreateNode { id, name } => {
                tree.insert(id, TargetKind::Element { name });
            }
            DriverDomCommand::CreateText { id, value } => {
                tree.insert(id, TargetKind::Text { value });
            }
            DriverDomCommand::CreateComment { id, value } => {
                tree.insert(id, TargetKind::Comment { value });
            }
            DriverDomCommand::UpdateText { id, value } => {
                if let Some(node) = tree.nodes.get_mut(&id) {
                    node.kind = TargetKind::Text { value };
                }
            }
            DriverDomCommand::SetAttr { id, name, value } => {
                if let Some(node) = tree.nodes.get_mut(&id) {
                    node.attrs.insert(name, value);
                }
            }
            DriverDomCommand::RemoveAttr { id, name } => {
                if let Some(node) = tree.nodes.get_mut(&id) {
                    node.attrs.remove(&name);
                }
            }
            DriverDomCommand::InsertBefore {
                parent,
                child,
                ref_id,
            } => {
                tree.unlink(child);
                tree.link(parent, child, ref_id);
            }
            DriverDomCommand::RemoveNode { id }
            | DriverDomCommand::RemoveText { id }
            | DriverDomCommand::RemoveComment { id } => {
                tree.unlink(id);
                tree.nodes.remove(&id);
            }
            other => passthrough.push(other),
        }
    }

    SplitBuffer { tree, passthrough }
}

impl TargetTree {
    fn insert(&mut self, id: DomId, kind: TargetKind) {
        self.nodes.insert(
            id,
            TargetNode {
                kind,
                attrs: BTreeMap::new(),
                children: Vec::new(),
            },
        );
    }

    fn unlink(&mut self, child: DomId) {
        let Some(parent) = self.parent.remove(&child) else {
            return;
        };

        if let Some(node) = self.nodes.get_mut(&parent)
            && let Some(at) = node.children.iter().position(|item| *item == child)
        {
            node.children.remove(at);
        }
    }

    fn link(&mut self, parent: DomId, child: DomId, ref_id: Option<DomId>) {
        let Some(node) = self.nodes.get_mut(&parent) else {
            return;
        };

        match ref_id.and_then(|ref_id| node.children.iter().position(|item| *item == ref_id)) {
            Some(at) => node.children.insert(at, child),
            None => node.children.push(child),
        }

        self.parent.insert(child, parent);
    }

    pub fn get(&self, id: DomId) -> Option<&TargetNode> {
        self.nodes.get(&id)
    }

    pub fn children(&self, id: DomId) -> &[DomId] {
        match self.nodes.get(&id) {
            Some(node) => node.children.as_slice(),
            None => &[],
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: u64) -> DomId {
        DomId::from_u64(value)
    }

    fn element(value: u64, name: &'static str) -> DriverDomCommand {
        DriverDomCommand::CreateNode {
            id: id(value),
            name: name.into(),
        }
    }

    fn insert(parent: u64, child: u64) -> DriverDomCommand {
        DriverDomCommand::InsertBefore {
            parent: id(parent),
            child: id(child),
            ref_id: None,
        }
    }

    #[test]
    fn builds_a_tree_from_creates_and_inserts() {
        let split = split_buffer(vec![
            element(3, "body"),
            element(4, "div"),
            insert(3, 4),
            element(5, "span"),
            insert(4, 5),
        ]);

        assert_eq!(split.tree.children(id(3)), &[id(4)]);
        assert_eq!(split.tree.children(id(4)), &[id(5)]);
        assert!(split.passthrough.is_empty());
    }

    /// `InsertBefore` with `ref_id` inserts before the referenced sibling, not at the end.
    #[test]
    fn insert_before_respects_the_reference() {
        let split = split_buffer(vec![
            element(3, "body"),
            element(4, "div"),
            insert(3, 4),
            element(5, "span"),
            DriverDomCommand::InsertBefore {
                parent: id(3),
                child: id(5),
                ref_id: Some(id(4)),
            },
        ]);

        assert_eq!(split.tree.children(id(3)), &[id(5), id(4)]);
    }

    /// Without detaching from the previous parent, a moved node would appear in two places
    /// and the matcher would stumble on a copy that does not exist in the DOM. The server
    /// does the same during replay (`AllElements::insert_before` calls `remove_from_parent`).
    #[test]
    fn a_moved_node_leaves_its_previous_parent() {
        let split = split_buffer(vec![
            element(3, "body"),
            element(4, "div"),
            element(5, "div"),
            element(6, "span"),
            insert(3, 4),
            insert(3, 5),
            insert(4, 6),
            insert(5, 6),
        ]);

        assert_eq!(split.tree.children(id(4)), &[] as &[DomId]);
        assert_eq!(split.tree.children(id(5)), &[id(6)]);
    }

    /// `CreateText` may carry a stale value: `Computed` read during an open transaction
    /// returns the cached value, so `DomText::patched` can emit old text and correct it
    /// immediately after. What matters is how the node ends the batch.
    #[test]
    fn update_text_wins_over_create_text() {
        let split = split_buffer(vec![
            DriverDomCommand::CreateText {
                id: id(4),
                value: "stale".to_string(),
            },
            DriverDomCommand::UpdateText {
                id: id(4),
                value: "fresh".to_string(),
            },
        ]);

        let Some(node) = split.tree.get(id(4)) else {
            panic!("the text node should be in the tree");
        };

        assert_eq!(
            node.kind,
            TargetKind::Text {
                value: "fresh".to_string()
            }
        );
    }

    #[test]
    fn attributes_are_accumulated_and_removed() {
        let split = split_buffer(vec![
            element(4, "div"),
            DriverDomCommand::SetAttr {
                id: id(4),
                name: "class".into(),
                value: "row".to_string(),
            },
            DriverDomCommand::SetAttr {
                id: id(4),
                name: "href".into(),
                value: "/a".to_string(),
            },
            DriverDomCommand::SetAttr {
                id: id(4),
                name: "class".into(),
                value: "col".to_string(),
            },
            DriverDomCommand::RemoveAttr {
                id: id(4),
                name: "href".into(),
            },
        ]);

        let Some(node) = split.tree.get(id(4)) else {
            panic!("the element should be in the tree");
        };

        assert_eq!(node.attrs.len(), 1);
        assert_eq!(
            node.attrs.get(&StaticString::from("class")),
            Some(&"col".to_string())
        );
    }

    /// A node created and removed within a single mount does not exist.
    #[test]
    fn a_node_created_and_removed_is_gone() {
        let split = split_buffer(vec![
            element(3, "body"),
            element(4, "div"),
            insert(3, 4),
            DriverDomCommand::RemoveNode { id: id(4) },
        ]);

        assert!(split.tree.get(id(4)).is_none());
        assert_eq!(split.tree.children(id(3)), &[] as &[DomId]);
    }

    /// CSS and callbacks are unrelated to node identity, so they pass through unchanged.
    /// Callbacks are keyed by `DomId`, which hydration preserves.
    #[test]
    fn css_and_callbacks_pass_through_in_order() {
        let split = split_buffer(vec![
            element(4, "div"),
            DriverDomCommand::InsertCss {
                selector: Some(".a".to_string()),
                value: "color:red".to_string(),
            },
            DriverDomCommand::CallbackAdd {
                id: id(4),
                event_name: "click".to_string(),
                callback_id: crate::dev::CallbackId::from_u64(7),
            },
        ]);

        assert_eq!(split.passthrough.len(), 2);
        assert!(matches!(
            split.passthrough[0],
            DriverDomCommand::InsertCss { .. }
        ));
        assert!(matches!(
            split.passthrough[1],
            DriverDomCommand::CallbackAdd { .. }
        ));
    }
}
