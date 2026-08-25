//! How many DOM commands one update emits.
//!
//! The browser suite in `tests/dom-bench` measures how long these updates take; this pins
//! down what they *do*, which is deterministic and needs no browser. A change that makes a
//! keystroke replace a node instead of patching it, or makes a list re-render rows it used
//! to move, shows up here as a failing count rather than as a slow benchmark nobody ran.
//!
//! Counts include the marker comments vertigo places to anchor reactive regions: a
//! `render_list` row is preceded by its own anchor comment, and each `render_value` leaves
//! one behind.

use std::collections::BTreeMap;

use crate::{self as vertigo, dom};
use crate::{
    DomNode, DomText, Value,
    dev::{
        command::DriverDomCommand,
        inspect::{log_start, log_take},
    },
    render::render_list,
};

fn variant(command: &DriverDomCommand) -> &'static str {
    match command {
        DriverDomCommand::CreateNode { .. } => "CreateNode",
        DriverDomCommand::CreateText { .. } => "CreateText",
        DriverDomCommand::UpdateText { .. } => "UpdateText",
        DriverDomCommand::SetAttr { .. } => "SetAttr",
        DriverDomCommand::RemoveAttr { .. } => "RemoveAttr",
        DriverDomCommand::RemoveNode { .. } => "RemoveNode",
        DriverDomCommand::RemoveText { .. } => "RemoveText",
        DriverDomCommand::InsertBefore { .. } => "InsertBefore",
        DriverDomCommand::InsertCss { .. } => "InsertCss",
        DriverDomCommand::CreateComment { .. } => "CreateComment",
        DriverDomCommand::RemoveComment { .. } => "RemoveComment",
        DriverDomCommand::CallbackAdd { .. } => "CallbackAdd",
        DriverDomCommand::CallbackRemove { .. } => "CallbackRemove",
    }
}

/// Commands emitted by `update`, with the mount excluded.
///
/// The tree is built before the log starts, so what comes back is the steady-state cost of
/// the update rather than the cost of creating the thing it updates. `_root` is held for
/// the duration: dropping a `DomNode` emits removals.
fn commands_for(root: impl FnOnce() -> DomNode, update: impl FnOnce()) -> BTreeMap<String, u32> {
    let _root = root();

    log_start();
    update();
    let commands = log_take();

    let mut counts: BTreeMap<String, u32> = BTreeMap::new();
    for command in &commands {
        *counts.entry(variant(command).to_string()).or_default() += 1;
    }
    counts
}

fn counts(pairs: &[(&str, u32)]) -> BTreeMap<String, u32> {
    pairs
        .iter()
        .map(|(name, count)| ((*name).to_string(), *count))
        .collect()
}

/// Interpolating a value into `dom!` patches the text node it already has.
///
/// This used to route through `render_value`, which replaced the whole node on every
/// change - three commands, a new id each time, and a marker comment left behind. The
/// `editor-keystroke-*` pair in `tests/dom-bench` measures what that cost.
#[test]
fn interpolated_text_is_patched_in_place() {
    let text = Value::new("aaa".to_string());

    let emitted = commands_for(
        || {
            let text = text.clone();
            dom! { <div>{text}</div> }
        },
        || text.set("bbb".to_string()),
    );

    assert_eq!(emitted, counts(&[("UpdateText", 1)]));
}

/// Reaching for [`DomText::new_computed`] by hand gets the same thing - it is what the
/// interpolation above is built on.
#[test]
fn computed_text_node_is_patched_in_place() {
    let text = Value::new("aaa".to_string());

    let emitted = commands_for(
        || {
            let node: DomNode = DomText::new_computed(text.clone()).into();
            dom! { <div>{node}</div> }
        },
        || text.set("bbb".to_string()),
    );

    assert_eq!(emitted, counts(&[("UpdateText", 1)]));
}

/// Mounting is a single `CreateText` carrying the value, not an empty node patched
/// afterwards.
///
/// This is load-bearing rather than cosmetic: server-side rendering replays these commands
/// into HTML, and the hydration pass builds its virtual tree from `CreateText` while
/// ignoring `UpdateText`. A node created empty would hydrate empty.
#[test]
fn interpolated_text_mounts_with_its_value_already_set() {
    let text = Value::new("hello".to_string());

    log_start();
    let _root = {
        let text = text.clone();
        dom! { <div>{text}</div> }
    };
    let commands = log_take();

    let created: Vec<&String> = commands
        .iter()
        .filter_map(|command| match command {
            DriverDomCommand::CreateText { value, .. } => Some(value),
            _ => None,
        })
        .collect();

    assert_eq!(created, vec!["hello"], "{commands:?}");
    assert!(
        !commands
            .iter()
            .any(|command| matches!(command, DriverDomCommand::UpdateText { .. })),
        "mounting must not need a follow-up patch: {commands:?}"
    );
}

/// A class change is one attribute write, with no node churn.
#[test]
fn attribute_change_is_one_command() {
    let class = Value::new("row".to_string());

    let emitted = commands_for(
        || {
            let class = class.clone();
            dom! { <div class={class} /> }
        },
        || class.set("row sel".to_string()),
    );

    assert_eq!(emitted, counts(&[("SetAttr", 1)]));
}

/// A write whose value does not change anything downstream must not reach the DOM at all.
#[test]
fn a_cut_off_write_emits_nothing() {
    let caret = Value::new(0u32);
    // Constant while the caret stays inside one run - the shape the editor toolbar has.
    let active_block = caret.map(|offset| offset / 4_096);

    let emitted = commands_for(
        || dom! { <div class={active_block.map(|index| index.to_string())} /> },
        || caret.set(37),
    );

    assert!(
        emitted.is_empty(),
        "a write absorbed by the equality cutoff must emit no DOM commands, got {emitted:?}"
    );
}

fn keys(range: std::ops::Range<u32>) -> Vec<u32> {
    range.collect()
}

fn mount_list(order: &Value<Vec<u32>>) -> DomNode {
    let list = render_list(
        order,
        |key| *key,
        |key| key.render_value(|key| dom! { <li>{key}</li> }),
    );
    dom! { <ul>{list}</ul> }
}

/// Appending one row costs the same whether the list holds ten rows or a thousand: the
/// reconciler's longest-common-prefix phase absorbs every row that stayed put.
#[test]
fn append_cost_does_not_depend_on_list_length() {
    let measure = |size: u32| {
        let order = Value::new(keys(0..size));
        commands_for(|| mount_list(&order), || order.set(keys(0..size + 1)))
    };

    let small = measure(10);
    let large = measure(1_000);

    assert_eq!(small, large, "append cost must not scale with list length");
    // One row: its anchor comment, the element, its text, and the marker its body leaves.
    assert_eq!(
        small,
        counts(&[
            ("CreateComment", 2),
            ("CreateNode", 1),
            ("CreateText", 1),
            ("InsertBefore", 4),
        ])
    );
}

/// Removing from the middle costs the same as removing from the end - the prefix and suffix
/// phases meet around the gap, and only the anchor differs.
#[test]
fn removal_cost_does_not_depend_on_position() {
    let from_end = {
        let order = Value::new(keys(0..100));
        commands_for(|| mount_list(&order), || order.set(keys(0..99)))
    };

    let from_middle = {
        let order = Value::new(keys(0..100));
        commands_for(
            || mount_list(&order),
            || {
                let mut without = keys(0..100);
                without.remove(50);
                order.set(without);
            },
        )
    };

    assert_eq!(from_end, from_middle);
}

/// A full reverse is the reconciler's worst case - prefix and suffix match nothing, so every
/// row goes through the keyed middle map. Rows must be *moved*, never rebuilt.
///
/// One row short of every row, in fact: no two rows of a reversed list are in ascending order
/// relative to each other, so the longest run the middle phase can leave alone is a single
/// row. That one is the floor, and it is what tells reordering apart from rebuilding.
///
/// The per-row cost is one insert per node the row spans, because a marker that is moved
/// brings the content it reports along with it. Here a row is three nodes - its anchor
/// comment, the `render_value` marker, and the element that marker owns. A row whose body is
/// a plain element rather than a `render_value` spans two, which is what the
/// `list-reverse` workload in `tests/dom-bench` measures.
#[test]
fn reversing_moves_every_row_but_one_and_rebuilds_none() {
    let size = 50u32;
    let order = Value::new(keys(0..size));

    let emitted = commands_for(
        || mount_list(&order),
        || {
            let mut backward = keys(0..size);
            backward.reverse();
            order.set(backward);
        },
    );

    assert_eq!(
        emitted,
        counts(&[("InsertBefore", 3 * (size - 1))]),
        "reversing must only re-insert - no row may be recreated"
    );
}

/// Swapping two rows moves two rows, whatever the list length.
///
/// The reconciler's prefix and suffix phases only strip one row from each end here, so
/// everything between the two swapped positions goes through the keyed middle map.
#[test]
fn swap_cost_does_not_depend_on_the_distance_between_the_rows() {
    let measure = |size: u32| {
        let order = Value::new(keys(0..size));
        commands_for(
            || {
                let list = render_list(
                    &order,
                    |key| *key,
                    |key| {
                        let key = crate::transaction(|ctx| key.get(ctx));
                        dom! { <li>{key}</li> }
                    },
                );
                dom! { <ul>{list}</ul> }
            },
            || {
                let mut swapped = keys(0..size);
                swapped.swap(1, (size - 2) as usize);
                order.set(swapped);
            },
        )
    };

    let near = measure(100);
    let far = measure(1_000);

    assert_eq!(near, far, "swap cost must not scale with list length");
    // Two rows, two nodes each - the row's anchor comment and its element.
    assert_eq!(near, counts(&[("InsertBefore", 4)]));
}

/// The same swap over `render_value` rows: still two rows, but three nodes each.
#[test]
fn swapping_render_value_rows_moves_two_rows() {
    let size = 1_000u32;
    let order = Value::new(keys(0..size));

    let emitted = commands_for(
        || mount_list(&order),
        || {
            let mut swapped = keys(0..size);
            swapped.swap(1, (size - 2) as usize);
            order.set(swapped);
        },
    );

    assert_eq!(emitted, counts(&[("InsertBefore", 6)]));
}

/// The same reverse over rows that are plain elements: two nodes per row, so two inserts.
///
/// Pinned separately from the `render_value` case above so that a change to how markers
/// carry their content cannot pass by shifting both numbers together.
#[test]
fn reversing_plain_element_rows_moves_two_nodes_each_but_one_row() {
    let size = 50u32;
    let order = Value::new(keys(0..size));

    let emitted = commands_for(
        || {
            let list = render_list(
                &order,
                |key| *key,
                |key| {
                    let key = crate::transaction(|ctx| key.get(ctx));
                    dom! { <li>{key}</li> }
                },
            );
            dom! { <ul>{list}</ul> }
        },
        || {
            let mut backward = keys(0..size);
            backward.reverse();
            order.set(backward);
        },
    );

    assert_eq!(emitted, counts(&[("InsertBefore", 2 * (size - 1))]));
}

/// Changing one row's value notifies that row alone. The list's own subscription does not
/// fire, so no reordering work happens at all.
#[test]
fn updating_one_row_does_not_disturb_the_list() {
    let rows: Vec<Value<String>> = (0..20).map(|i| Value::new(format!("row {i}"))).collect();
    let order = Value::new(keys(0..20));
    let cells = rows.clone();

    let emitted = commands_for(
        || {
            let list = render_list(
                &order,
                |key| *key,
                move |key| {
                    let key = crate::transaction(|ctx| key.get(ctx));
                    let cell = cells[key as usize].clone();
                    dom! { <li>{cell}</li> }
                },
            );
            dom! { <ul>{list}</ul> }
        },
        || rows[10].set("changed".to_string()),
    );

    assert_eq!(
        emitted,
        counts(&[("UpdateText", 1)]),
        "one row's update must not touch the other rows"
    );
}
