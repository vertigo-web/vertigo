use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
    rc::Rc,
};

use crate::{
    Computed, DomComment, DomNode, KeyedListItem, ToComputed, dom::dom_id::DomId,
    driver_module::get_driver_dom, keyed_computed_list, struct_mut::ValueMut,
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

    DomComment::new_marker("list element", move |parent_id, comment_id, content| {
        let current_list: Rc<ValueMut<VecDeque<(K, Row)>>> =
            Rc::new(ValueMut::new(VecDeque::new()));

        Some(rows.clone().subscribe({
            let render = render.clone();
            let content = content.clone();

            move |new_list| {
                current_list.change(|current| {
                    let prev = std::mem::take(current);
                    *current = reorder_nodes(
                        parent_id,
                        comment_id,
                        prev,
                        VecDeque::from(new_list),
                        render.as_ref(),
                    );

                    content.set(
                        current
                            .iter()
                            .flat_map(|(_, row)| [row.anchor_id(), row.node.id_dom()])
                            .collect(),
                    );
                })
            }
        }))
    })
    .into()
}

/// One rendered row of the list.
///
/// A row is not necessarily a single sibling under the list parent:
/// [`render_value`](crate::Computed::render_value) keeps its content *in front of*
/// its own marker and re-creates it every time that marker is mounted. `anchor` is
/// an empty comment kept directly in front of the row, so that whatever shape the
/// row has, it always has one stable node marking where it begins — that is what
/// insert and move operations anchor on.
struct Row {
    anchor: DomComment,
    node: DomNode,
}

impl Row {
    fn new(node: DomNode) -> Row {
        Row {
            anchor: DomComment::new("row"),
            node,
        }
    }

    fn anchor_id(&self) -> DomId {
        self.anchor.id_dom()
    }

    /// Insert the row - or move it, when it is already mounted - in front of `before`.
    fn insert_before(&self, parent_id: DomId, before: DomId) {
        let driver = get_driver_dom();

        driver.insert_before(parent_id, self.anchor.id_dom(), Some(before));
        // Mounting the node renders its content in front of itself, which lands
        // between the anchor and the node.
        driver.insert_before(parent_id, self.node.id_dom(), Some(before));
    }
}

fn reorder_nodes<T: Clone + PartialEq + 'static, K: Clone + Eq + Hash>(
    parent_id: DomId,
    comment_id: DomId,
    mut real_child: VecDeque<(K, Row)>,
    mut new_child: VecDeque<KeyedListItem<K, Computed<T>>>,
    render: &dyn Fn(&Computed<T>) -> DomNode,
) -> VecDeque<(K, Row)> {
    let pairs_top = get_pairs_top(&mut real_child, &mut new_child);
    let mut pairs_bottom = get_pairs_bottom(&mut real_child, &mut new_child);

    let last_before = pairs_bottom
        .front()
        .map(|(_, row)| row.anchor_id())
        .unwrap_or(comment_id);

    let mut pairs_middle = get_pairs_middle(parent_id, last_before, real_child, new_child, render);

    let mut pairs = pairs_top;
    pairs.append(&mut pairs_middle);
    pairs.append(&mut pairs_bottom);
    pairs
}

fn get_pairs_top<T: Clone + PartialEq, K: PartialEq>(
    current: &mut VecDeque<(K, Row)>,
    new_child: &mut VecDeque<KeyedListItem<K, Computed<T>>>,
) -> VecDeque<(K, Row)> {
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

fn get_pairs_bottom<T: Clone + PartialEq, K: PartialEq>(
    current: &mut VecDeque<(K, Row)>,
    new_child: &mut VecDeque<KeyedListItem<K, Computed<T>>>,
) -> VecDeque<(K, Row)> {
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

fn get_pairs_middle<T: Clone + PartialEq + 'static, K: Clone + Eq + Hash>(
    parent_id: DomId,
    last_before: DomId,
    real_child: VecDeque<(K, Row)>,
    new_child: VecDeque<KeyedListItem<K, Computed<T>>>,
    render: &dyn Fn(&Computed<T>) -> DomNode,
) -> VecDeque<(K, Row)> {
    let mut cache: HashMap<K, Row> = real_child.into_iter().collect();
    let mut pairs_middle = VecDeque::new();

    for item in new_child {
        let row = match cache.remove(&item.key) {
            Some(row) => row,
            None => Row::new(render(&item.value)),
        };

        row.insert_before(parent_id, last_before);
        pairs_middle.push_back((item.key, row));
    }

    pairs_middle
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use super::{Row, render_list, reorder_nodes};
    use crate::{self as vertigo, dom};
    use crate::{
        Computed, DomId, DomNode, KeyedListItem, Value,
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
        format!("<!-- row --><li>{label}</li><!-- v -->")
    }

    fn list_html(rows: &[&str]) -> String {
        format!(
            "<ul>{rows}<!-- list element --></ul>",
            rows = rows.iter().map(|label| row_html(label)).collect::<String>()
        )
    }

    /// A row whose root is a plain element (no `render_value` wrapper around it).
    fn mount_element_list(items: &Value<Vec<(u32, String)>>) -> DomNode {
        let list = render_list(
            items,
            |item| item.0,
            |item| {
                let label = item.map(|item| item.1);
                dom! { <li>{label}</li> }
            },
        );
        dom! { <ul>{list}</ul> }
    }

    fn element_pseudo_html_after(
        items: &Value<Vec<(u32, String)>>,
        update: impl FnOnce(),
    ) -> String {
        log_start();
        let _root = mount_element_list(items);
        update();
        DomDebugFragment::from_log().to_pseudo_html()
    }

    /// A row whose body is an interpolated reactive value. That embeds as a self-patching
    /// text node, so - unlike the `render_value` rows above - it leaves no marker comment
    /// inside the element.
    fn element_list_html(rows: &[&str]) -> String {
        format!(
            "<ul>{rows}<!-- list element --></ul>",
            rows = rows
                .iter()
                .map(|label| format!("<!-- row --><li>{label}</li>"))
                .collect::<String>()
        )
    }

    #[test]
    fn element_rows_prepend() {
        let items = Value::new(vec![row(2, "two"), row(3, "three")]);

        let html = element_pseudo_html_after(&items, || {
            items.set(vec![row(1, "one"), row(2, "two"), row(3, "three")]);
        });

        assert_eq!(html, element_list_html(&["one", "two", "three"]));
    }

    #[test]
    fn element_rows_insert_in_middle() {
        let items = Value::new(vec![row(1, "one"), row(3, "three")]);

        let html = element_pseudo_html_after(&items, || {
            items.set(vec![row(1, "one"), row(2, "two"), row(3, "three")]);
        });

        assert_eq!(html, element_list_html(&["one", "two", "three"]));
    }

    /// The trailing row is untouched, so the moved rows have to anchor on it
    /// instead of on the list marker.
    #[test]
    fn element_rows_reorder_in_middle() {
        let items = Value::new(vec![
            row(1, "one"),
            row(2, "two"),
            row(3, "three"),
            row(4, "four"),
        ]);

        let html = element_pseudo_html_after(&items, || {
            items.set(vec![
                row(1, "one"),
                row(3, "three"),
                row(2, "two"),
                row(4, "four"),
            ]);
        });

        assert_eq!(html, element_list_html(&["one", "three", "two", "four"]));
    }

    /// A row only ever anchors on nodes it owns, so unrelated siblings under the
    /// same parent keep their place.
    #[test]
    fn sibling_before_the_list_keeps_its_place() {
        let items = Value::new(vec![row(2, "two"), row(3, "three"), row(4, "four")]);

        log_start();
        let list = render_list(
            &items,
            |item| item.0,
            |item| item.render_value(|item| dom! { <li>{item.1.as_str()}</li> }),
        );
        let _root = dom! { <ul><li>"header"</li>{list}</ul> };
        items.set(vec![
            row(1, "one"),
            row(3, "three"),
            row(2, "two"),
            row(4, "four"),
        ]);

        let rows = ["one", "three", "two", "four"]
            .iter()
            .map(|label| row_html(label))
            .collect::<String>();

        assert_eq!(
            DomDebugFragment::from_log().to_pseudo_html(),
            format!("<ul><li>header</li>{rows}<!-- list element --></ul>")
        );
    }

    /// Two lists interleaved under one parent: each moves only its own rows.
    #[test]
    fn two_lists_in_the_same_parent_do_not_interfere() {
        let left = Value::new(vec![row(1, "l1"), row(2, "l2")]);
        let right = Value::new(vec![row(1, "r1"), row(2, "r2"), row(3, "r3")]);

        log_start();
        let left_list = render_list(
            &left,
            |item| item.0,
            |item| item.render_value(|item| dom! { <li>{item.1.as_str()}</li> }),
        );
        let right_list = render_list(
            &right,
            |item| item.0,
            |item| item.render_value(|item| dom! { <li>{item.1.as_str()}</li> }),
        );
        let _root = dom! { <ul>{left_list}{right_list}</ul> };

        right.set(vec![row(2, "r2"), row(1, "r1"), row(3, "r3")]);

        let left_rows = ["l1", "l2"]
            .iter()
            .map(|label| row_html(label))
            .collect::<String>();
        let right_rows = ["r2", "r1", "r3"]
            .iter()
            .map(|label| row_html(label))
            .collect::<String>();

        assert_eq!(
            DomDebugFragment::from_log().to_pseudo_html(),
            format!("<ul>{left_rows}<!-- list element -->{right_rows}<!-- list element --></ul>")
        );
    }

    /// A row that is itself a list spans several siblings of the outer list's
    /// parent, none of which is at a fixed offset from the row's own node.
    #[test]
    fn row_that_is_itself_a_list_reorders() {
        let items = Value::new(vec![row(1, "one"), row(2, "two"), row(3, "three")]);

        log_start();
        let list = render_list(
            &items,
            |item| item.0,
            |item| {
                let labels = item.map(|item| vec![item.1]);
                render_list(
                    labels,
                    |label| label.clone(),
                    |label| label.render_value(|label| dom! { <li>{label}</li> }),
                )
            },
        );
        let _root = dom! { <ul>{list}</ul> };
        items.set(vec![row(2, "two"), row(1, "one"), row(3, "three")]);

        let rows = ["two", "one", "three"]
            .iter()
            .map(|label| {
                format!("<!-- row --><!-- row --><li>{label}</li><!-- v --><!-- list element -->")
            })
            .collect::<String>();

        assert_eq!(
            DomDebugFragment::from_log().to_pseudo_html(),
            format!("<ul>{rows}<!-- list element --></ul>")
        );
    }

    /// A key appearing during an update runs `render` while the graph is mid-refresh, so
    /// the nested list this row renders is built at that point.
    #[test]
    fn row_that_is_itself_a_list_can_be_added_during_an_update() {
        let items = Value::new(vec![row(1, "one")]);

        log_start();
        let list = render_list(
            &items,
            |item| item.0,
            |item| {
                let labels = item.map(|item| vec![item.1]);
                render_list(
                    labels,
                    |label| label.clone(),
                    |label| label.render_value(|label| dom! { <li>{label}</li> }),
                )
            },
        );
        let _root = dom! { <ul>{list}</ul> };
        items.set(vec![row(1, "one"), row(2, "two")]);

        let rows = ["one", "two"]
            .iter()
            .map(|label| {
                format!("<!-- row --><!-- row --><li>{label}</li><!-- v --><!-- list element -->")
            })
            .collect::<String>();

        assert_eq!(
            DomDebugFragment::from_log().to_pseudo_html(),
            format!("<ul>{rows}<!-- list element --></ul>")
        );
    }

    #[test]
    fn key_removed_and_added_again() {
        let items = Value::new(vec![row(1, "one"), row(2, "two"), row(3, "three")]);

        let html = pseudo_html_after(&items, || {
            items.set(vec![row(1, "one"), row(3, "three")]);
            items.set(vec![row(1, "one"), row(2, "two"), row(3, "three")]);
        });

        assert_eq!(html, list_html(&["one", "two", "three"]));
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

    /// Moving a row must carry its DOM along, not rebuild it — a rebuilt row loses
    /// focus, selection, scroll position and running animations.
    #[test]
    fn moving_a_row_keeps_its_content() {
        let items = Value::new(vec![
            row(1, "one"),
            row(2, "two"),
            row(3, "three"),
            row(4, "four"),
        ]);
        let content_renders = Rc::new(Cell::new(0));

        log_start();
        let list = render_list(&items, |item| item.0, {
            let content_renders = content_renders.clone();
            move |item| {
                let content_renders = content_renders.clone();
                item.render_value(move |item| {
                    content_renders.set(content_renders.get() + 1);
                    dom! { <li>{item.1.as_str()}</li> }
                })
            }
        });
        let _root = dom! { <ul>{list}</ul> };
        assert_eq!(content_renders.get(), 4);

        items.set(vec![
            row(1, "one"),
            row(3, "three"),
            row(2, "two"),
            row(4, "four"),
        ]);

        assert_eq!(
            content_renders.get(),
            4,
            "moving a row must not re-render its content"
        );
        assert_eq!(
            DomDebugFragment::from_log().to_pseudo_html(),
            list_html(&["one", "three", "two", "four"])
        );
    }

    /// A moved row keeps its subscription, so a later value change still lands in the
    /// row's new position rather than where it used to be.
    #[test]
    fn updating_a_moved_row_replaces_content_in_place() {
        let items = Value::new(vec![
            row(1, "one"),
            row(2, "two"),
            row(3, "three"),
            row(4, "four"),
        ]);

        let html = pseudo_html_after(&items, || {
            items.set(vec![
                row(1, "one"),
                row(3, "three"),
                row(2, "two"),
                row(4, "four"),
            ]);
            items.set(vec![
                row(1, "one"),
                row(3, "three"),
                row(2, "TWO"),
                row(4, "four"),
            ]);
        });

        assert_eq!(html, list_html(&["one", "three", "TWO", "four"]));
    }

    /// The same guarantee one level down: moving a row whose node is a nested marker
    /// must not rebuild the nested subtree either.
    #[test]
    fn moving_a_row_keeps_nested_content() {
        let items = Value::new(vec![
            row(1, "one"),
            row(2, "two"),
            row(3, "three"),
            row(4, "four"),
        ]);
        let content_renders = Rc::new(Cell::new(0));

        log_start();
        let list = render_list(&items, |item| item.0, {
            let content_renders = content_renders.clone();
            move |item| {
                let label = item.map(|item| item.1);
                let content_renders = content_renders.clone();
                item.render_value(move |_| {
                    let content_renders = content_renders.clone();
                    label.render_value(move |label| {
                        content_renders.set(content_renders.get() + 1);
                        dom! { <li>{label}</li> }
                    })
                })
            }
        });
        let _root = dom! { <ul>{list}</ul> };
        assert_eq!(content_renders.get(), 4);

        items.set(vec![
            row(1, "one"),
            row(3, "three"),
            row(2, "two"),
            row(4, "four"),
        ]);

        assert_eq!(
            content_renders.get(),
            4,
            "moving a row must not re-render its nested content"
        );
    }

    /// A row that occupies more than two siblings: the inner `render_value` adds
    /// its own marker, so the row is `[anchor][content][inner marker][outer marker]`.
    #[test]
    fn nested_render_value_rows_reorder() {
        let items = Value::new(vec![
            row(1, "one"),
            row(2, "two"),
            row(3, "three"),
            row(4, "four"),
        ]);

        log_start();
        let list = render_list(
            &items,
            |item| item.0,
            |item| {
                let label = item.map(|item| item.1);
                item.render_value(move |_| label.render_value(|label| dom! { <li>{label}</li> }))
            },
        );
        let _root = dom! { <ul>{list}</ul> };
        items.set(vec![
            row(1, "one"),
            row(3, "three"),
            row(2, "two"),
            row(4, "four"),
        ]);

        let rows = ["one", "three", "two", "four"]
            .iter()
            .map(|label| format!("<!-- row --><li>{label}</li><!-- v --><!-- v -->"))
            .collect::<String>();

        assert_eq!(
            DomDebugFragment::from_log().to_pseudo_html(),
            format!("<ul>{rows}<!-- list element --></ul>")
        );
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
            "<ul><!-- row --><li>one</li><!-- row --><li>two</li><!-- row --><li>three</li><!-- list element --></ul>"
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
        let existing = Row::new(DomNode::from("same"));
        let node_id = existing.node.id_dom();
        let render_calls = Rc::new(std::cell::Cell::new(0usize));

        let result = reorder_nodes(
            DomId::from_u64(200),
            DomId::from_u64(201),
            std::collections::VecDeque::from([(1, existing)]),
            std::collections::VecDeque::from([item(1, "same")]),
            &{
                let render_calls = render_calls.clone();
                move |value: &Computed<String>| {
                    render_calls.set(render_calls.get() + 1);
                    crate::transaction(|ctx| DomNode::from(value.get(ctx)))
                }
            },
        );

        assert_eq!(result.len(), 1);
        assert_eq!(render_calls.get(), 0);
        assert_eq!(result[0].1.node.id_dom(), node_id);
    }

    #[test]
    fn renders_only_the_new_key() {
        let existing = Row::new(DomNode::from("old"));
        let existing_id = existing.node.id_dom();
        let render_calls = Rc::new(std::cell::Cell::new(0usize));

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
        );

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1.node.id_dom(), existing_id);
        assert_eq!(result[1].0, 2);
        assert_eq!(render_calls.get(), 1);
    }
}
