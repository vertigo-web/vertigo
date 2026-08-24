//! A list widget: N rows, each with its own label, quantity and class.

use std::{collections::HashMap, rc::Rc};

use vertigo::{DomNode, Value, dom, render::render_list, transaction};

pub const ITEMS: u32 = 500;
/// A second, much shorter list, so "append cost does not depend on list size" has something
/// to be compared against rather than merely asserted about one size.
pub const ITEMS_SMALL: u32 = 50;

pub struct RowState {
    pub label: Value<String>,
    pub qty: Value<u32>,
    pub class: Value<String>,
}

pub struct ListScene {
    /// Membership and order. This, and only this, drives the reconciler.
    pub order: Value<Vec<u32>>,
    /// Per-row state, deliberately kept *out* of the list's value type.
    ///
    /// The obvious alternative - a `Value<Vec<Item>>` - would make editing one row rebuild
    /// the whole vector and re-run the keyed diff before a single DOM command is emitted,
    /// and that graph work would drown the DOM work this suite exists to measure.
    pub rows: Rc<HashMap<u32, RowState>>,
    /// The canonical full order, `0..n`.
    pub keys: Vec<u32>,
    /// A key that starts outside the list, with its row state already built, so the append
    /// and insert workloads only ever mutate `order` inside the timed loop.
    pub spare: u32,
    /// Two labels, and two classes, of equal length - see the module docs in `workloads.rs`
    /// on why alternating values must not change size.
    pub labels: [String; 2],
    pub classes: [String; 2],
}

pub fn build(count: u32) -> Rc<ListScene> {
    let spare = count;
    let mut rows = HashMap::new();

    for key in 0..=spare {
        rows.insert(
            key,
            RowState {
                label: Value::new(format!("Item {key:04}")),
                qty: Value::new(1),
                class: Value::new("row".to_string()),
            },
        );
    }

    Rc::new(ListScene {
        order: Value::new((0..count).collect()),
        rows: Rc::new(rows),
        keys: (0..count).collect(),
        spare,
        labels: ["Item aaaa".to_string(), "Item bbbb".to_string()],
        classes: ["row".to_string(), "row sel".to_string()],
    })
}

pub fn render(scene: Rc<ListScene>) -> DomNode {
    let rows = scene.rows.clone();

    let list = render_list(
        &scene.order,
        |key| *key,
        move |key| {
            let key = transaction(|ctx| key.get(ctx));
            let Some(row) = rows.get(&key) else {
                return dom! { <div class="row missing" /> };
            };
            dom! {
                <div class={row.class.clone()}>
                    <span class="k">{key}</span>
                    <span class="l">{row.label.clone()}</span>
                    <span class="q">{row.qty.clone()}</span>
                </div>
            }
        },
    );

    dom! { <div id="stage-list">{list}</div> }
}
