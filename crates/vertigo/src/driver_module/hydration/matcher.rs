use std::collections::HashSet;

use super::{
    report::HydrationReport,
    snapshot::{DomSnapshot, SnapshotNode},
    target_tree::{SplitBuffer, TargetKind, TargetNode, TargetTree},
};
use crate::{dev::command::DriverDomCommand, dom::dom_id::DomId, driver_module::StaticString};

pub(crate) struct Reconciled {
    pub commands: Vec<DriverDomCommand>,
    pub report: HydrationReport,
}

/// Reconciles the tree that the application built with the browser's DOM state.
///
/// The result is a reduced stream: adoptions of existing nodes, creation of only what
/// the server didn't render, patches only where something differs, and removal of leftovers.
pub(crate) fn reconcile(split: SplitBuffer, snapshot: &DomSnapshot) -> Reconciled {
    let SplitBuffer { tree, passthrough } = split;

    let report = HydrationReport {
        total: tree.len() as u64,
        ..HydrationReport::default()
    };

    // Without a `<body>` there's nowhere to start the walk. Instead of adopting anything
    // blindly, we fall back to a stream without hydration - the same decision as when
    // the batch has no body to start from.
    let Some(body) = snapshot.body else {
        let mut commands = rebuild_verbatim(&tree);
        commands.extend(passthrough);

        return Reconciled { commands, report };
    };

    let mut matcher = Matcher {
        tree: &tree,
        snapshot,
        out: Vec::new(),
        report: HydrationReport {
            root_found: true,
            ..report
        },
    };

    if !snapshot.nodes.is_empty() {
        matcher.reconcile_attrs(DomId::root_id(), 0);
    }

    matcher.reconcile_root(DomId::body(), body);

    if let Some(head) = snapshot.head {
        matcher.reconcile_root(DomId::head(), head);
    }

    let mut commands = matcher.out;
    commands.extend(passthrough);

    Reconciled {
        commands,
        report: matcher.report,
    }
}

/// Variant for `--disable-hydration`: nothing is adopted, the server's content is
/// removed, and the mount buffer goes out unchanged.
pub(crate) fn discard(split: SplitBuffer, snapshot: &DomSnapshot) -> Vec<DriverDomCommand> {
    let SplitBuffer { tree, passthrough } = split;

    let mut commands = Vec::new();

    for root in [snapshot.body, snapshot.head].into_iter().flatten() {
        for child in snapshot.children(root) {
            commands.push(DriverDomCommand::SnapshotRemove { snapshot: *child });
        }
    }

    commands.extend(rebuild_verbatim(&tree));
    commands.extend(passthrough);
    commands
}

/// Mount buffer in the form it would have without hydration.
///
/// `split_buffer` consumed the original structural commands, so both paths that need
/// to build the tree from scratch - missing `<body>` in the snapshot and
/// `--disable-hydration` - reconstruct them from the index. Order is document order:
/// parent before children.
fn rebuild_verbatim(tree: &TargetTree) -> Vec<DriverDomCommand> {
    let mut out = Vec::new();
    let mut visited: HashSet<DomId> = HashSet::new();

    for id in [DomId::root_id(), DomId::head(), DomId::body()] {
        if let Some(node) = tree.get(id) {
            out.extend(create_commands(id, node));
        }
    }

    // Three entries, not one, because `<html>` might not be in the tree - an app mounted
    // via `start_app` without its own `<html>` gets just `<head>` and `<body>`. The visited
    // set ensures that `<head>` and `<body>` reached from `<html>` aren't walked twice.
    for root in [DomId::root_id(), DomId::head(), DomId::body()] {
        rebuild_children(tree, root, &mut visited, &mut out);
    }

    out
}

fn rebuild_children(
    tree: &TargetTree,
    parent: DomId,
    visited: &mut HashSet<DomId>,
    out: &mut Vec<DriverDomCommand>,
) {
    if !visited.insert(parent) {
        return;
    }

    for child in tree.children(parent) {
        let child = *child;

        // Document roots already exist - `MapNodes` resolves their ids dynamically.
        if !child.is_document_root()
            && let Some(node) = tree.get(child)
        {
            out.extend(create_commands(child, node));
        }

        out.push(DriverDomCommand::InsertBefore {
            parent,
            child,
            ref_id: None,
        });

        rebuild_children(tree, child, visited, out);
    }
}

fn create_commands(id: DomId, node: &TargetNode) -> Vec<DriverDomCommand> {
    let mut out = Vec::new();

    match &node.kind {
        TargetKind::Element { name } => {
            out.push(DriverDomCommand::CreateNode {
                id,
                name: name.clone(),
            });
        }
        TargetKind::Text { value } => out.push(DriverDomCommand::CreateText {
            id,
            value: value.clone(),
        }),
        TargetKind::Comment { value } => out.push(DriverDomCommand::CreateComment {
            id,
            value: value.clone(),
        }),
    }

    for (name, value) in &node.attrs {
        out.push(DriverDomCommand::SetAttr {
            id,
            name: name.clone(),
            value: value.clone(),
        });
    }

    out
}

/// What's decided about one target child in the first pass.
///
/// Two passes, not one, because `InsertBefore` for a created node needs the identifier
/// of the **next** adopted sibling - and that's knowledge about the future if we were
/// emitting while deciding.
enum ChildPlan {
    Adopt {
        child: DomId,
        snapshot: u32,
    },
    AdoptText {
        child: DomId,
        snapshot: u32,
        patch: bool,
    },
    Create {
        child: DomId,
    },
}

impl ChildPlan {
    fn child(&self) -> DomId {
        match self {
            Self::Adopt { child, .. } | Self::AdoptText { child, .. } | Self::Create { child } => {
                *child
            }
        }
    }

    fn is_adopted(&self) -> bool {
        !matches!(self, Self::Create { .. })
    }
}

struct Matcher<'a> {
    tree: &'a TargetTree,
    snapshot: &'a DomSnapshot,
    out: Vec<DriverDomCommand>,
    report: HydrationReport,
}

impl<'a> Matcher<'a> {
    fn reconcile_root(&mut self, root: DomId, snapshot_index: u32) {
        self.reconcile_attrs(root, snapshot_index);
        self.reconcile_children(root, snapshot_index);
    }

    fn reconcile_attrs(&mut self, id: DomId, snapshot_index: u32) {
        let tree = self.tree;
        let snapshot = self.snapshot;

        let Some(node) = tree.get(id) else {
            return;
        };

        let existing = snapshot.attrs(snapshot_index);

        for attr in existing {
            let wanted = node.attrs.keys().any(|key| key.as_str() == attr.name);
            if !wanted {
                self.out.push(DriverDomCommand::RemoveAttr {
                    id,
                    name: StaticString::from(attr.name.clone()),
                });
            }
        }

        for (name, value) in &node.attrs {
            let same = existing
                .iter()
                .any(|attr| attr.name == name.as_str() && &attr.value == value);

            if !same {
                self.out.push(DriverDomCommand::SetAttr {
                    id,
                    name: name.clone(),
                    value: value.clone(),
                });
            }
        }
    }

    fn reconcile_children(&mut self, parent: DomId, parent_snapshot: u32) {
        let (plans, removals) = self.plan_children(parent, parent_snapshot);
        self.emit_children(parent, &plans);

        for snapshot in removals {
            self.out.push(DriverDomCommand::SnapshotRemove { snapshot });
        }
    }

    /// First pass: who adopts which snapshot node, and who gets created from scratch.
    fn plan_children(&mut self, parent: DomId, parent_snapshot: u32) -> (Vec<ChildPlan>, Vec<u32>) {
        let tree = self.tree;
        let snapshot = self.snapshot;

        let target_children = tree.children(parent);
        let snapshot_children = snapshot.children(parent_snapshot);

        let mut plans = Vec::with_capacity(target_children.len());
        let mut removals = Vec::new();
        let mut cursor = 0usize;
        let mut index = 0usize;

        while index < target_children.len() {
            let child = target_children[index];

            let Some(node) = tree.get(child) else {
                index += 1;
                continue;
            };

            match &node.kind {
                // `render_value`/`render_list` markers have no html counterpart: the server
                // cuts comments (`get_render_child_mode` rejects `HtmlNode::Comment`).
                // So they have nothing to match and don't count against the result.
                TargetKind::Comment { .. } => {
                    self.report.skipped += 1;
                    plans.push(ChildPlan::Create { child });
                    index += 1;
                }
                TargetKind::Element { name } => {
                    self.report.hydratable += 1;

                    let mut from = cursor;
                    while snapshot_children
                        .get(from)
                        .is_some_and(|index| is_whitespace(snapshot, *index))
                    {
                        from += 1;
                    }

                    match find_element(snapshot, snapshot_children, from, name.as_str()) {
                        Some(found) => {
                            removals.extend_from_slice(&snapshot_children[cursor..found]);
                            let snapshot_index = snapshot_children[found];
                            plans.push(ChildPlan::Adopt {
                                child,
                                snapshot: snapshot_index,
                            });
                            cursor = found + 1;
                        }
                        None => plans.push(ChildPlan::Create { child }),
                    }

                    index += 1;
                }
                TargetKind::Text { value } => {
                    let run = text_run_length(tree, target_children, index);
                    let taken = self.plan_text_run(
                        &mut plans,
                        target_children,
                        index,
                        run,
                        snapshot_children,
                        cursor,
                        value,
                    );
                    cursor = taken;
                    index += run;
                }
            }
        }

        removals.extend_from_slice(&snapshot_children[cursor.min(snapshot_children.len())..]);

        (plans, removals)
    }

    /// Second pass: emission, in three rounds over the same siblings.
    ///
    /// The order of rounds is not cosmetic. `InsertBefore` for a created node points to
    /// an adopted sibling as its reference point, and `MapNodes` will only resolve that
    /// identifier once the adoption has registered it - so all sibling adoptions must
    /// go out before the first insert. Descent into depth goes last, because it emits
    /// commands about different siblings and has no effect on this level.
    fn emit_children(&mut self, parent: DomId, plans: &[ChildPlan]) {
        for plan in plans {
            match plan {
                ChildPlan::Adopt { child, snapshot } => {
                    self.out.push(DriverDomCommand::NodeAdopt {
                        id: *child,
                        snapshot: *snapshot,
                    });
                    self.report.matched += 1;
                    self.reconcile_attrs(*child, *snapshot);
                }
                ChildPlan::AdoptText {
                    child,
                    snapshot,
                    patch,
                } => {
                    self.out.push(DriverDomCommand::NodeAdopt {
                        id: *child,
                        snapshot: *snapshot,
                    });
                    self.report.matched += 1;

                    if *patch {
                        let tree = self.tree;

                        if let Some(node) = tree.get(*child)
                            && let TargetKind::Text { value } = &node.kind
                        {
                            self.out.push(DriverDomCommand::UpdateText {
                                id: *child,
                                value: value.clone(),
                            });
                        }
                    }
                }
                ChildPlan::Create { .. } => {}
            }
        }

        for (at, plan) in plans.iter().enumerate() {
            if let ChildPlan::Create { child } = plan {
                let ref_id = plans
                    .iter()
                    .skip(at + 1)
                    .find(|plan| plan.is_adopted())
                    .map(ChildPlan::child);

                self.create_subtree(*child, parent, ref_id);
            }
        }

        for plan in plans {
            if let ChildPlan::Adopt { child, snapshot } = plan {
                self.reconcile_children(*child, *snapshot);
            }
        }
    }

    fn create_subtree(&mut self, id: DomId, parent: DomId, ref_id: Option<DomId>) {
        let tree = self.tree;

        if let Some(node) = tree.get(id) {
            self.out.extend(create_commands(id, node));
        }

        self.out.push(DriverDomCommand::InsertBefore {
            parent,
            child: id,
            ref_id,
        });

        for child in tree.children(id) {
            self.create_subtree(*child, id, None);
        }
    }

    /// Plans a run of `run` consecutive text children, starting at `index`.
    ///
    /// Merged-run detection happens on the target tree side, not by analyzing snapshot
    /// content: the server merges exactly such runs, so the run's length is sufficient
    /// and we don't need to match prefixes.
    ///
    /// Returns the new cursor position in `snapshot_children`.
    #[allow(clippy::too_many_arguments)]
    fn plan_text_run(
        &mut self,
        plans: &mut Vec<ChildPlan>,
        target_children: &[DomId],
        index: usize,
        run: usize,
        snapshot_children: &[u32],
        cursor: usize,
        value: &str,
    ) -> usize {
        // Only the first in the run has a chance of matching. The rest were merged by
        // the server, so they have nothing to match - just like comment markers, and
        // they likewise don't count against the result.
        self.report.hydratable += 1;
        self.report.skipped += (run - 1) as u64;

        let first = target_children[index];

        let matched = match snapshot_children.get(cursor) {
            Some(snapshot_index) if is_text(self.snapshot, *snapshot_index) => {
                let patch = run > 1 || !text_equals(self.snapshot, *snapshot_index, value);

                plans.push(ChildPlan::AdoptText {
                    child: first,
                    snapshot: *snapshot_index,
                    patch,
                });

                cursor + 1
            }
            _ => {
                plans.push(ChildPlan::Create { child: first });
                cursor
            }
        };

        for child in target_children.iter().skip(index + 1).take(run - 1) {
            plans.push(ChildPlan::Create { child: *child });
        }

        matched
    }
}

/// Element name, case-insensitive and without the `svg:` prefix.
///
/// JS sends `tagName` lowercased, and case-insensitive comparison is correct for both
/// html and svg at once - which saves carrying the sixty-element `SVG_TAGS` set into
/// wasm. `tags.ts` stays in js because `createElement` needs it anyway to choose the
/// namespace.
fn tag_matches(target: &str, candidate: &str) -> bool {
    let local = target.strip_prefix("svg:").unwrap_or(target);
    local.eq_ignore_ascii_case(candidate)
}

fn find_element(
    snapshot: &DomSnapshot,
    children: &[u32],
    from: usize,
    name: &str,
) -> Option<usize> {
    for (at, index) in children.iter().enumerate().skip(from) {
        if let Some(SnapshotNode::Element {
            name: candidate, ..
        }) = snapshot.node(*index)
            && tag_matches(name, candidate)
        {
            return Some(at);
        }
    }

    None
}

fn text_run_length(tree: &TargetTree, children: &[DomId], from: usize) -> usize {
    let mut length = 0;

    for child in children.iter().skip(from) {
        match tree.get(*child) {
            Some(node) if matches!(node.kind, TargetKind::Text { .. }) => length += 1,
            _ => break,
        }
    }

    length.max(1)
}

fn is_text(snapshot: &DomSnapshot, index: u32) -> bool {
    matches!(snapshot.node(index), Some(SnapshotNode::Text { .. }))
}

fn is_whitespace(snapshot: &DomSnapshot, index: u32) -> bool {
    match snapshot.node(index) {
        Some(SnapshotNode::Text { value }) => value.trim().is_empty(),
        _ => false,
    }
}

fn text_equals(snapshot: &DomSnapshot, index: u32, value: &str) -> bool {
    matches!(snapshot.node(index), Some(SnapshotNode::Text { value: existing }) if existing == value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver_module::hydration::{SnapshotAttr, split_buffer};

    fn id(value: u64) -> DomId {
        DomId::from_u64(value)
    }

    fn element(value: u64, name: &'static str) -> DriverDomCommand {
        DriverDomCommand::CreateNode {
            id: id(value),
            name: name.into(),
        }
    }

    fn text(value: u64, content: &str) -> DriverDomCommand {
        DriverDomCommand::CreateText {
            id: id(value),
            value: content.to_string(),
        }
    }

    fn insert(parent: u64, child: u64) -> DriverDomCommand {
        DriverDomCommand::InsertBefore {
            parent: id(parent),
            child: id(child),
            ref_id: None,
        }
    }

    fn attr(id_value: u64, name: &'static str, value: &str) -> DriverDomCommand {
        DriverDomCommand::SetAttr {
            id: id(id_value),
            name: name.into(),
            value: value.to_string(),
        }
    }

    fn snap_element(name: &str, children: Vec<u32>) -> SnapshotNode {
        SnapshotNode::Element {
            name: name.to_string(),
            attrs: vec![],
            children,
        }
    }

    fn snap_element_attrs(
        name: &str,
        attrs: Vec<(&str, &str)>,
        children: Vec<u32>,
    ) -> SnapshotNode {
        SnapshotNode::Element {
            name: name.to_string(),
            attrs: attrs
                .into_iter()
                .map(|(name, value)| SnapshotAttr {
                    name: name.to_string(),
                    value: value.to_string(),
                })
                .collect(),
            children,
        }
    }

    fn snap_text(value: &str) -> SnapshotNode {
        SnapshotNode::Text {
            value: value.to_string(),
        }
    }

    /// Snapshot shaped like `<html><head/><body>{body_children}</body></html>`, where
    /// body nodes start at index 3.
    fn document(body_children: Vec<SnapshotNode>) -> DomSnapshot {
        let count = body_children.len() as u32;
        let mut nodes = vec![
            snap_element("html", vec![1, 2]),
            snap_element("head", vec![]),
            snap_element("body", (0..count).map(|offset| offset + 3).collect()),
        ];
        nodes.extend(body_children);

        DomSnapshot {
            nodes,
            head: Some(1),
            body: Some(2),
        }
    }

    /// Minimal mount buffer: `<html>`, `<head>`, `<body>` and what's below.
    fn mount_buffer(below_body: Vec<DriverDomCommand>) -> Vec<DriverDomCommand> {
        let mut commands = vec![
            element(1, "html"),
            element(2, "head"),
            insert(1, 2),
            element(3, "body"),
            insert(1, 3),
        ];
        commands.extend(below_body);
        commands
    }

    fn run(below_body: Vec<DriverDomCommand>, snapshot: &DomSnapshot) -> Reconciled {
        reconcile(split_buffer(mount_buffer(below_body)), snapshot)
    }

    fn adopted(commands: &[DriverDomCommand]) -> Vec<(u64, u32)> {
        commands
            .iter()
            .filter_map(|command| match command {
                DriverDomCommand::NodeAdopt { id, snapshot } => Some((id.to_u64(), *snapshot)),
                _ => None,
            })
            .collect()
    }

    fn removed(commands: &[DriverDomCommand]) -> Vec<u32> {
        commands
            .iter()
            .filter_map(|command| match command {
                DriverDomCommand::SnapshotRemove { snapshot } => Some(*snapshot),
                _ => None,
            })
            .collect()
    }

    fn created(commands: &[DriverDomCommand]) -> Vec<u64> {
        commands
            .iter()
            .filter_map(|command| match command {
                DriverDomCommand::CreateNode { id, .. }
                | DriverDomCommand::CreateText { id, .. }
                | DriverDomCommand::CreateComment { id, .. } => Some(id.to_u64()),
                _ => None,
            })
            .collect()
    }

    /// A node that the server already rendered is adopted, not created anew.
    #[test]
    fn a_matching_element_is_adopted() {
        let snapshot = document(vec![snap_element("div", vec![])]);

        let result = run(vec![element(4, "div"), insert(3, 4)], &snapshot);

        assert_eq!(adopted(&result.commands), vec![(4, 3)]);
        assert!(created(&result.commands).is_empty());
        assert_eq!(result.report.matched, 1);
        assert_eq!(result.report.hydratable, 1);
        assert!(result.report.root_found);
    }

    /// An adopted node that's already in place doesn't generate an `InsertBefore`.
    /// This is where the batch shrinkage comes from.
    #[test]
    fn an_adopted_node_in_place_needs_no_insert() {
        let snapshot = document(vec![
            snap_element("div", vec![]),
            snap_element("span", vec![]),
        ]);

        let result = run(
            vec![
                element(4, "div"),
                insert(3, 4),
                element(5, "span"),
                insert(3, 5),
            ],
            &snapshot,
        );

        assert_eq!(adopted(&result.commands), vec![(4, 3), (5, 4)]);
        assert!(
            !result
                .commands
                .iter()
                .any(|command| matches!(command, DriverDomCommand::InsertBefore { .. })),
            "nothing moved, so nothing should be inserted: {:?}",
            result.commands
        );
    }

    /// Matching descends into children.
    #[test]
    fn matching_recurses_into_children() {
        let snapshot = document(vec![snap_element("div", vec![4]), snap_text("hello")]);

        let result = run(
            vec![
                element(4, "div"),
                insert(3, 4),
                text(5, "hello"),
                insert(4, 5),
            ],
            &snapshot,
        );

        assert_eq!(adopted(&result.commands), vec![(4, 3), (5, 4)]);
    }

    /// Attributes are reconciled in both directions. The server could have rendered an
    /// attribute that the client tree doesn't have - it renders from the same stream,
    /// but a component might draw something different under `is_browser()`, and then
    /// a leftover `href` would survive on a node that already belongs to the browser.
    #[test]
    fn attributes_are_reconciled_in_both_directions() {
        let snapshot = document(vec![snap_element_attrs(
            "a",
            vec![("href", "/server"), ("title", "stays")],
            vec![],
        )]);

        let result = run(
            vec![
                element(4, "a"),
                attr(4, "title", "stays"),
                attr(4, "class", "new"),
                insert(3, 4),
            ],
            &snapshot,
        );

        let sets: Vec<(&str, &str)> = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DriverDomCommand::SetAttr { name, value, .. } => {
                    Some((name.as_str(), value.as_str()))
                }
                _ => None,
            })
            .collect();

        let removes: Vec<&str> = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DriverDomCommand::RemoveAttr { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect();

        assert_eq!(sets, vec![("class", "new")], "only the difference is sent");
        assert_eq!(removes, vec!["href"], "the server's leftover goes away");
    }

    /// A node without a counterpart is created from scratch and inserted before the
    /// next adopted sibling.
    #[test]
    fn an_unmatched_target_is_created_before_the_next_adopted_sibling() {
        let snapshot = document(vec![snap_element("span", vec![])]);

        let result = run(
            vec![
                element(4, "div"),
                insert(3, 4),
                element(5, "span"),
                insert(3, 5),
            ],
            &snapshot,
        );

        assert_eq!(adopted(&result.commands), vec![(5, 3)]);
        assert_eq!(created(&result.commands), vec![4]);

        let inserts: Vec<(u64, u64, Option<u64>)> = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DriverDomCommand::InsertBefore {
                    parent,
                    child,
                    ref_id,
                } => Some((
                    parent.to_u64(),
                    child.to_u64(),
                    ref_id.map(|id| id.to_u64()),
                )),
                _ => None,
            })
            .collect();

        assert_eq!(inserts, vec![(3, 4, Some(5))]);
    }

    /// Lack of a match is not evidence that the snapshot's content is garbage: we create
    /// the node, but don't move the cursor or remove anything *at this step*. The snapshot's
    /// `<span>` only dies at the end, as a leftover that nobody claimed - not as a node
    /// skipped on the way to a hit.
    #[test]
    fn a_missing_match_leaves_the_cursor_where_it_was() {
        let snapshot = document(vec![snap_element("span", vec![])]);

        let result = run(vec![element(4, "div"), insert(3, 4)], &snapshot);

        assert!(created(&result.commands).contains(&4));
        assert_eq!(
            removed(&result.commands),
            vec![3],
            "the span is a leftover only because no target child ever claimed it"
        );
    }

    /// Nodes skipped on the way to a hit are removed, one command per root of a
    /// discarded subtree.
    ///
    /// Snapshot built directly here, because `document` hangs all passed nodes under
    /// `<body>`, and here we need nesting.
    #[test]
    fn skipped_nodes_are_removed_once_per_subtree() {
        let snapshot = DomSnapshot {
            nodes: vec![
                snap_element("html", vec![1, 2]),
                snap_element("head", vec![]),
                snap_element("body", vec![3, 5]),
                snap_element("p", vec![4]),
                snap_text("deep inside the p"),
                snap_element("div", vec![]),
            ],
            head: Some(1),
            body: Some(2),
        };

        let result = run(vec![element(4, "div"), insert(3, 4)], &snapshot);

        assert_eq!(adopted(&result.commands), vec![(4, 5)]);
        assert_eq!(
            removed(&result.commands),
            vec![3],
            "removing the <p> takes its subtree with it - the text inside needs no command \
             of its own"
        );
    }

    /// A snapshot without a `<body>` means hydration has nowhere to start its walk.
    /// We fall back to a stream without hydration: the whole tree is built from scratch,
    /// nothing is adopted, and `root_found` says so explicitly.
    #[test]
    fn a_snapshot_without_a_body_falls_back_to_building_everything() {
        let snapshot = DomSnapshot {
            nodes: vec![snap_element("html", vec![])],
            head: None,
            body: None,
        };

        let result = run(vec![element(4, "div"), insert(3, 4)], &snapshot);

        assert!(!result.report.root_found);
        assert!(adopted(&result.commands).is_empty());
        assert!(removed(&result.commands).is_empty());
        assert!(
            created(&result.commands).contains(&4),
            "the app still has to be built: {:?}",
            result.commands
        );
    }

    /// Attributes of the document roots are reconciled even though the roots themselves
    /// are not adopted - `MapNodes` resolves ids 1, 2 and 3 dynamically.
    #[test]
    fn the_document_roots_are_not_adopted_but_their_attributes_are() {
        let mut snapshot = document(vec![]);
        snapshot.nodes[0] = snap_element_attrs("html", vec![("lang", "en")], vec![1, 2]);

        let mut below = vec![attr(1, "lang", "pl")];
        below.push(attr(3, "class", "page"));

        let result = run(below, &snapshot);

        assert!(adopted(&result.commands).is_empty());

        let sets: Vec<(u64, &str, &str)> = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DriverDomCommand::SetAttr { id, name, value } => {
                    Some((id.to_u64(), name.as_str(), value.as_str()))
                }
                _ => None,
            })
            .collect();

        assert!(sets.contains(&(1, "lang", "pl")));
        assert!(sets.contains(&(3, "class", "page")));
    }

    /// The passthrough group makes it to the result untouched.
    #[test]
    fn passthrough_commands_are_kept() {
        let snapshot = document(vec![snap_element("div", vec![])]);

        let result = run(
            vec![
                element(4, "div"),
                insert(3, 4),
                DriverDomCommand::CallbackAdd {
                    id: id(4),
                    event_name: "click".to_string(),
                    callback_id: crate::dev::CallbackId::from_u64(7),
                },
                DriverDomCommand::InsertCss {
                    selector: None,
                    value: "a{}".to_string(),
                },
            ],
            &snapshot,
        );

        assert!(
            result
                .commands
                .iter()
                .any(|command| matches!(command, DriverDomCommand::CallbackAdd { .. }))
        );
        assert!(
            result
                .commands
                .iter()
                .any(|command| matches!(command, DriverDomCommand::InsertCss { .. }))
        );
    }

    /// With hydration disabled we don't adopt anything, remove the server's content, and
    /// send the buffer unchanged. Today this corresponds to `removeInitNodes` in js.
    #[test]
    fn discard_removes_the_server_markup_and_keeps_the_buffer() {
        let snapshot = document(vec![snap_element("div", vec![]), snap_text("hello")]);

        let commands = discard(split_buffer(mount_buffer(vec![])), &snapshot);

        assert_eq!(removed(&commands), vec![3, 4]);
        assert!(adopted(&commands).is_empty());
        assert!(created(&commands).contains(&3), "the buffer is untouched");
    }

    /// Multiple adjacent `DomText` nodes are merged by the server into a single text run
    /// (`last_text_add` in `get_render_child_mode`), and the parser creates one node from it.
    ///
    /// The merged node adopts the first text and receives an `UpdateText` trimming it to its
    /// own value; siblings are created anew. The js equivalent counted the rest as matched
    /// without binding them to anything - and a later `UpdateText` on the second would hit
    /// an id absent from `MapNodes`.
    #[test]
    fn a_merged_text_run_is_split() {
        let snapshot = document(vec![snap_text("firstsecond")]);

        let result = run(
            vec![
                text(4, "first"),
                insert(3, 4),
                text(5, "second"),
                insert(3, 5),
            ],
            &snapshot,
        );

        assert_eq!(adopted(&result.commands), vec![(4, 3)]);
        assert_eq!(created(&result.commands), vec![5]);

        let patches: Vec<(u64, &str)> = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DriverDomCommand::UpdateText { id, value } => Some((id.to_u64(), value.as_str())),
                _ => None,
            })
            .collect();

        assert_eq!(
            patches,
            vec![(4, "first")],
            "the adopted node is cut down to its own value"
        );

        assert_eq!(
            result.report.skipped, 1,
            "the second text cannot be matched - the server merged it away - so it must not \
             count against the score"
        );
        assert_eq!(result.report.hydratable, 1);
        assert_eq!(result.report.matched, 1);
    }

    /// A single text with matching content needs no patch.
    #[test]
    fn an_identical_single_text_is_adopted_without_a_patch() {
        let snapshot = document(vec![snap_text("hello")]);

        let result = run(vec![text(4, "hello"), insert(3, 4)], &snapshot);

        assert_eq!(adopted(&result.commands), vec![(4, 3)]);
        assert!(
            !result
                .commands
                .iter()
                .any(|command| matches!(command, DriverDomCommand::UpdateText { .. })),
            "nothing changed, so nothing should be patched"
        );
    }

    #[test]
    fn a_differing_single_text_is_adopted_and_patched() {
        let snapshot = document(vec![snap_text("stale")]);

        let result = run(vec![text(4, "fresh"), insert(3, 4)], &snapshot);

        assert_eq!(adopted(&result.commands), vec![(4, 3)]);

        let patches: Vec<&str> = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DriverDomCommand::UpdateText { value, .. } => Some(value.as_str()),
                _ => None,
            })
            .collect();

        assert_eq!(patches, vec!["fresh"]);
    }

    /// Production server rendering formats output (`convert_to_string(true)`), so the html
    /// contains indentation that the parser turns into text nodes not present in the vertigo
    /// tree. When searching for an element, we're allowed to skip and remove them.
    ///
    /// This is safe because contexts don't overlap: where whitespace is significant
    /// (`<pre>`, inline content), formatting switches to `Format::none()` and injects
    /// nothing - and then the corresponding text node also exists in the target tree.
    #[test]
    fn formatting_whitespace_is_skipped_when_looking_for_an_element() {
        let snapshot = document(vec![
            snap_text("\n  "),
            snap_element("div", vec![]),
            snap_text("\n"),
        ]);

        let result = run(vec![element(4, "div"), insert(3, 4)], &snapshot);

        assert_eq!(adopted(&result.commands), vec![(4, 4)]);
        assert_eq!(removed(&result.commands), vec![3, 5]);
        assert!(created(&result.commands).is_empty());
    }

    /// SVG elements preserve their case in `tagName`, html reports uppercase.
    /// JS sends the name lowercased, and case-insensitive comparison handles both
    /// worlds at once - so `SVG_TAGS` doesn't need to reach wasm.
    #[test]
    fn svg_casing_matches_without_a_tag_table() {
        let snapshot = document(vec![
            snap_element("svg", vec![4]),
            snap_element("lineargradient", vec![]),
        ]);

        let result = run(
            vec![
                element(4, "svg"),
                insert(3, 4),
                element(5, "linearGradient"),
                insert(4, 5),
            ],
            &snapshot,
        );

        assert_eq!(adopted(&result.commands), vec![(4, 3), (5, 4)]);
    }

    /// A name with `svg:` prefix matches its local name - js creates such an element
    /// via `createElementNS` after stripping the prefix, so the parser sees `<a>`.
    #[test]
    fn an_svg_prefixed_name_matches_its_local_name() {
        let snapshot = document(vec![
            snap_element("svg", vec![4]),
            snap_element("a", vec![]),
        ]);

        let result = run(
            vec![
                element(4, "svg"),
                insert(3, 4),
                element(5, "svg:a"),
                insert(4, 5),
            ],
            &snapshot,
        );

        assert_eq!(adopted(&result.commands), vec![(4, 3), (5, 4)]);
    }

    /// A comment marker is created anew and doesn't consume a snapshot node - the next
    /// target element must hit what the server actually rendered.
    #[test]
    fn a_comment_marker_does_not_consume_a_snapshot_node() {
        let snapshot = document(vec![snap_element("div", vec![])]);

        let result = run(
            vec![
                DriverDomCommand::CreateComment {
                    id: id(4),
                    value: "marker".to_string(),
                },
                insert(3, 4),
                element(5, "div"),
                insert(3, 5),
            ],
            &snapshot,
        );

        assert_eq!(adopted(&result.commands), vec![(5, 3)]);
        assert_eq!(created(&result.commands), vec![4]);
        assert_eq!(result.report.skipped, 1);
        assert_eq!(result.report.hydratable, 1);
        assert!(removed(&result.commands).is_empty());
    }
}
