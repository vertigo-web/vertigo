use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
    rc::Rc,
};

use crate::{
    Computed, DomComment, DomNode, DropResource, KeyedListItem, ToComputed,
    computed::struct_mut::ValueMut, dev::command::DriverDomCommand, dom::dom_id::DomId,
    driver_module::get_driver_dom, keyed_computed_list,
};

/// Render an iterable as a keyed list of DOM nodes.
///
/// Each key keeps a stable [`Computed`](crate::Computed) (via [`keyed_computed_list`]).
/// `render` is called when a key **appears**; the node is dropped when the key
/// **leaves**. Item content updates go through that `Computed` — embed it in
/// `dom!`, or wrap with [`Computed::render_value`](crate::Computed::render_value).
///
/// Duplicate keys are skipped (first occurrence is kept).
///
/// ```rust
/// use vertigo::{dom, Value, render::render_list};
///
/// let my_list = Value::new(vec![
///     (1, "one"),
///     (2, "two"),
///     (3, "three"),
/// ]);
///
/// let elements = render_list(
///     &my_list.to_computed(),
///     |el| el.0,
///     |el| el.render_value(|el| dom! { <div>{el.1}</div> })
/// );
///
/// dom! {
///     <div>
///         {elements}
///     </div>
/// };
/// ```
pub fn render_list<
    T: Clone + PartialEq + 'static,
    K: Clone + Eq + Hash + std::fmt::Debug + 'static,
>(
    computed: impl ToComputed<Vec<T>>,
    get_key: impl Fn(&T) -> K + 'static,
    render: impl Fn(&Computed<T>) -> DomNode + 'static,
) -> DomNode {
    let rows = keyed_computed_list(computed, get_key);
    let render = Rc::new(render);

    DomComment::new_marker("list element", move |parent_id, comment_id| {
        let current_list: Rc<ValueMut<VecDeque<(K, DomNode)>>> =
            Rc::new(ValueMut::new(VecDeque::new()));
        let child_order = Rc::new(ValueMut::new(Vec::<DomId>::new()));

        let inspect = get_driver_dom().inspect_command({
            let child_order = child_order.clone();

            move |command| match command {
                DriverDomCommand::InsertBefore {
                    parent,
                    child,
                    ref_id,
                } if parent == parent_id => {
                    child_order.change(|order| update_child_order(order, child, ref_id));
                }
                DriverDomCommand::RemoveNode { id }
                | DriverDomCommand::RemoveText { id }
                | DriverDomCommand::RemoveComment { id } => {
                    child_order.change(|order| order.retain(|node_id| *node_id != id));
                }
                _ => {}
            }
        });

        let rows_sub = rows.clone().subscribe({
            let render = render.clone();
            let child_order = child_order.clone();

            move |new_list| {
                current_list.change(|current| {
                    let prev = std::mem::take(current);
                    *current = reorder_nodes(
                        parent_id,
                        comment_id,
                        prev,
                        VecDeque::from(new_list),
                        render.as_ref(),
                        child_order.as_ref(),
                    );
                })
            }
        });

        Some(DropResource::new(move || {
            inspect.off();
            rows_sub.off();
        }))
    })
    .into()
}

fn update_child_order(order: &mut Vec<DomId>, child: DomId, ref_id: Option<DomId>) {
    order.retain(|node_id| *node_id != child);

    if let Some(ref_id) = ref_id
        && let Some(index) = order.iter().position(|node_id| *node_id == ref_id)
    {
        order.insert(index, child);
        return;
    }

    order.push(child);
}

/// `render_value` rows are `[content…][marker]` siblings under the list parent.
/// For insert/move use the content node before the marker, not the marker itself.
fn row_content_before_marker(order: &[DomId], marker_id: DomId) -> Option<DomId> {
    order
        .iter()
        .position(|node_id| *node_id == marker_id)
        .and_then(|index| index.checked_sub(1))
        .map(|index| order[index])
}

fn find_insert_anchor<K>(
    pairs_bottom: &VecDeque<(K, DomNode)>,
    comment_id: DomId,
    child_order: &[DomId],
) -> DomId {
    pairs_bottom
        .front()
        .map(|(_, node)| node.id_dom())
        .and_then(|marker_id| row_content_before_marker(child_order, marker_id))
        .unwrap_or(comment_id)
}

fn reposition_row(parent_id: DomId, before: DomId, marker_id: DomId, child_order: &[DomId]) {
    if let Some(content_id) = row_content_before_marker(child_order, marker_id) {
        get_driver_dom().insert_before(parent_id, content_id, Some(before));
    }

    get_driver_dom().insert_before(parent_id, marker_id, Some(before));
}

fn reorder_nodes<T: Clone + 'static, K: Clone + Eq + Hash>(
    parent_id: DomId,
    comment_id: DomId,
    mut real_child: VecDeque<(K, DomNode)>,
    mut new_child: VecDeque<KeyedListItem<K, Computed<T>>>,
    render: &dyn Fn(&Computed<T>) -> DomNode,
    child_order: &ValueMut<Vec<DomId>>,
) -> VecDeque<(K, DomNode)> {
    let pairs_top = get_pairs_top(&mut real_child, &mut new_child);
    let mut pairs_bottom = get_pairs_bottom(&mut real_child, &mut new_child);

    let order = child_order.get();
    let last_before = find_insert_anchor(&pairs_bottom, comment_id, &order);
    let mut pairs_middle = get_pairs_middle(
        parent_id,
        last_before,
        real_child,
        new_child,
        render,
        child_order,
    );

    let mut pairs = pairs_top;
    pairs.append(&mut pairs_middle);
    pairs.append(&mut pairs_bottom);
    pairs
}

fn get_pairs_top<T: Clone, K: PartialEq>(
    current: &mut VecDeque<(K, DomNode)>,
    new_child: &mut VecDeque<KeyedListItem<K, Computed<T>>>,
) -> VecDeque<(K, DomNode)> {
    let mut pairs_top = VecDeque::new();

    loop {
        match (current.pop_front(), new_child.pop_front()) {
            (Some((key, node)), Some(item)) => {
                if key == item.key {
                    pairs_top.push_back((key, node));
                    continue;
                }

                current.push_front((key, node));
                new_child.push_front(item);
            }
            (Some(pair), None) => current.push_front(pair),
            (None, Some(item)) => new_child.push_front(item),
            (None, None) => {}
        }

        return pairs_top;
    }
}

fn get_pairs_bottom<T: Clone, K: PartialEq>(
    current: &mut VecDeque<(K, DomNode)>,
    new_child: &mut VecDeque<KeyedListItem<K, Computed<T>>>,
) -> VecDeque<(K, DomNode)> {
    let mut pairs_bottom = VecDeque::new();

    loop {
        match (current.pop_back(), new_child.pop_back()) {
            (Some((key, node)), Some(item)) => {
                if key == item.key {
                    pairs_bottom.push_front((key, node));
                    continue;
                }

                current.push_back((key, node));
                new_child.push_back(item);
            }
            (Some(pair), None) => current.push_back(pair),
            (None, Some(item)) => new_child.push_back(item),
            (None, None) => {}
        }

        return pairs_bottom;
    }
}

fn get_pairs_middle<T: Clone + 'static, K: Clone + Eq + Hash>(
    parent_id: DomId,
    last_before: DomId,
    real_child: VecDeque<(K, DomNode)>,
    new_child: VecDeque<KeyedListItem<K, Computed<T>>>,
    render: &dyn Fn(&Computed<T>) -> DomNode,
    child_order: &ValueMut<Vec<DomId>>,
) -> VecDeque<(K, DomNode)> {
    let mut cache: HashMap<K, DomNode> = real_child.into_iter().collect();
    let mut pairs_middle = VecDeque::new();

    for item in new_child {
        let node = match cache.remove(&item.key) {
            Some(node) => node,
            None => render(&item.value),
        };
        let marker_id = node.id_dom();
        let order = child_order.get();

        reposition_row(parent_id, last_before, marker_id, &order);
        pairs_middle.push_back((item.key, node));
    }

    pairs_middle
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use super::{render_list, reorder_nodes};
    use crate::{self as vertigo, dom};
    use crate::{
        Computed, DomId, DomNode, KeyedListItem, Value,
        computed::struct_mut::ValueMut,
        dev::inspect::{DomDebugFragment, log_start},
    };

    fn row(key: u32, label: &str) -> (u32, String) {
        (key, label.to_string())
    }

    fn item(key: u32, label: &str) -> KeyedListItem<u32, Computed<String>> {
        KeyedListItem {
            key,
            value: Computed::from({
                let label = label.to_string();
                move |_| label.clone()
            }),
        }
    }

    fn mount_render_value_list(items: &Value<Vec<(u32, String)>>) -> DomNode {
        let list = render_list(
            items,
            |item| item.0,
            |item| item.render_value(|item| dom! { <li>{item.1.as_str()}</li> }),
        );
        dom! { <ul>{list}</ul> }
    }

    fn pseudo_html(items: &Value<Vec<(u32, String)>>) -> String {
        log_start();
        let _root = mount_render_value_list(items);
        DomDebugFragment::from_log().to_pseudo_html()
    }

    fn pseudo_html_after(items: &Value<Vec<(u32, String)>>, update: impl FnOnce()) -> String {
        log_start();
        let _root = mount_render_value_list(items);
        update();
        DomDebugFragment::from_log().to_pseudo_html()
    }

    fn row_html(label: &str) -> String {
        format!("<li>{label}</li><!-- v -->")
    }

    fn list_html(rows: &[&str]) -> String {
        format!(
            "<ul>{rows}<!-- list element --></ul>",
            rows = rows.iter().map(|label| row_html(label)).collect::<String>()
        )
    }

    fn empty_child_order() -> ValueMut<Vec<DomId>> {
        ValueMut::new(Vec::new())
    }

    #[test]
    fn renders_three_items_in_source_order() {
        let items = Value::new(vec![row(1, "one"), row(2, "two"), row(3, "three")]);

        assert_eq!(pseudo_html(&items), list_html(&["one", "two", "three"]));
    }

    #[test]
    fn updates_item_content_without_rerendering() {
        let items = Value::new(vec![row(1, "one")]);
        let render_calls = Rc::new(Cell::new(0));

        log_start();
        let list = render_list(&items, |item| item.0, {
            let render_calls = render_calls.clone();
            move |item| {
                render_calls.set(render_calls.get() + 1);
                item.render_value(|item| dom! { <li>{item.1.as_str()}</li> })
            }
        });
        let _root = dom! { <ul>{list}</ul> };
        assert_eq!(render_calls.get(), 1);

        items.set(vec![row(1, "two")]);

        let html = DomDebugFragment::from_log().to_pseudo_html();
        assert_eq!(render_calls.get(), 1);
        assert_eq!(html, list_html(&["two"]));
    }

    #[test]
    fn appends_item_after_initial_render() {
        let items = Value::new(vec![row(1, "one"), row(2, "two")]);

        let html = pseudo_html_after(&items, || {
            items.set(vec![row(1, "one"), row(2, "two"), row(3, "three")]);
        });

        assert_eq!(html, list_html(&["one", "two", "three"]));
    }

    #[test]
    fn removes_item_from_middle() {
        let items = Value::new(vec![row(1, "one"), row(2, "two"), row(3, "three")]);

        let html = pseudo_html_after(&items, || {
            items.set(vec![row(1, "one"), row(3, "three")]);
        });

        assert_eq!(html, list_html(&["one", "three"]));
    }

    #[test]
    fn removes_all_items() {
        let items = Value::new(vec![row(1, "one"), row(2, "two")]);

        let html = pseudo_html_after(&items, || {
            items.set(Vec::new());
        });

        assert_eq!(html, "<ul><!-- list element --></ul>");
    }

    #[test]
    fn reorders_items_in_middle() {
        let items = Value::new(vec![row(1, "one"), row(2, "two"), row(3, "three")]);

        let html = pseudo_html_after(&items, || {
            items.set(vec![row(1, "one"), row(3, "three"), row(2, "two")]);
        });

        assert_eq!(html, list_html(&["one", "three", "two"]));
    }

    #[test]
    fn prepends_item() {
        let items = Value::new(vec![row(2, "two"), row(3, "three")]);

        let html = pseudo_html_after(&items, || {
            items.set(vec![row(1, "one"), row(2, "two"), row(3, "three")]);
        });

        assert_eq!(html, list_html(&["one", "two", "three"]));
    }

    #[test]
    fn renders_without_render_value_markers() {
        let items = Value::new(vec![row(1, "one"), row(2, "two"), row(3, "three")]);

        log_start();
        let list = render_list(
            &items,
            |item| item.0,
            |item| crate::transaction(|ctx| dom! { <li>{item.get(ctx).1.as_str()}</li> }),
        );
        let _root = dom! { <ul>{list}</ul> };

        assert_eq!(
            DomDebugFragment::from_log().to_pseudo_html(),
            "<ul><li>one</li><li>two</li><li>three</li><!-- list element --></ul>"
        );
    }

    #[test]
    fn renders_empty_list_then_populates() {
        let items = Value::new(Vec::<(u32, String)>::new());

        let html = pseudo_html_after(&items, || {
            items.set(vec![row(1, "one")]);
        });

        assert_eq!(html, list_html(&["one"]));
    }

    #[test]
    fn skips_duplicate_keys() {
        let items = Value::new(vec![row(1, "first"), row(1, "duplicate"), row(2, "two")]);

        assert_eq!(pseudo_html(&items), list_html(&["first", "two"]));
    }

    #[test]
    fn reuses_node_when_key_stays() {
        let node = DomNode::from("same");
        let node_id = node.id_dom();
        let render_calls = Rc::new(std::cell::Cell::new(0usize));

        let child_order = empty_child_order();

        let result = reorder_nodes(
            DomId::from_u64(200),
            DomId::from_u64(201),
            std::collections::VecDeque::from([(1, node)]),
            std::collections::VecDeque::from([item(1, "same")]),
            &{
                let render_calls = render_calls.clone();
                move |value: &Computed<String>| {
                    render_calls.set(render_calls.get() + 1);
                    crate::transaction(|ctx| DomNode::from(value.get(ctx)))
                }
            },
            &child_order,
        );

        assert_eq!(result.len(), 1);
        assert_eq!(render_calls.get(), 0);
        assert_eq!(result[0].1.id_dom(), node_id);
    }

    #[test]
    fn renders_only_the_new_key() {
        let existing = DomNode::from("old");
        let existing_id = existing.id_dom();
        let render_calls = Rc::new(std::cell::Cell::new(0usize));

        let child_order = empty_child_order();

        let result = reorder_nodes(
            DomId::from_u64(100),
            DomId::from_u64(101),
            std::collections::VecDeque::from([(1, existing)]),
            std::collections::VecDeque::from([item(1, "old"), item(2, "new")]),
            &{
                let render_calls = render_calls.clone();
                move |value: &Computed<String>| {
                    render_calls.set(render_calls.get() + 1);
                    crate::transaction(|ctx| DomNode::from(value.get(ctx)))
                }
            },
            &child_order,
        );

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1.id_dom(), existing_id);
        assert_eq!(result[1].0, 2);
        assert_eq!(render_calls.get(), 1);
    }
}
