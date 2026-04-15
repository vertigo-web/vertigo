use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
    rc::Rc,
};

use crate::{
    Computed, DomComment, DomNode, ToComputed, computed::struct_mut::ValueMut, dom::dom_id::DomId,
    driver_module::get_driver_dom,
};

/// Render iterable value (reactively transforms `Iterator<T>` into Node with list of rendered elements )
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
///     |el| dom! { <div>{el.1}</div> }
/// );
///
/// dom! {
///     <div>
///         {elements}
///     </div>
/// };
/// ```
///
///
pub fn render_list<
    T: PartialEq + Clone + 'static,
    K: Eq + Hash,
    L: IntoIterator<Item = T> + Clone + PartialEq + 'static,
>(
    computed: impl ToComputed<L>,
    get_key: impl Fn(&T) -> K + 'static,
    render: impl Fn(&T) -> DomNode + 'static,
) -> DomNode {
    let get_key = Rc::new(get_key);
    let render = Rc::new(render);

    let computed: Computed<L> = computed.to_computed();

    DomComment::new_marker("list element", move |parent_id, comment_id| {
        let current_list: Rc<ValueMut<VecDeque<(T, DomNode)>>> =
            Rc::new(ValueMut::new(VecDeque::new()));

        Some(computed.clone().subscribe({
            let get_key = get_key.clone();
            let render = render.clone();

            move |new_list| {
                let new_list = VecDeque::from_iter(new_list);
                current_list.change({
                    let get_key = get_key.clone();
                    let render = render.clone();

                    move |current| {
                        let current_list = std::mem::take(current);

                        let new_order = reorder_nodes(
                            parent_id,
                            comment_id,
                            current_list,
                            new_list,
                            get_key.clone(),
                            render,
                        );

                        *current = new_order;
                    }
                })
            }
        }))
    })
    .into()
}

fn reorder_nodes<T: PartialEq + Clone, K: Eq + Hash>(
    parent_id: DomId,
    comment_id: DomId,
    mut real_child: VecDeque<(T, DomNode)>,
    mut new_child: VecDeque<T>,
    get_key: Rc<dyn Fn(&T) -> K + 'static>,
    render: Rc<dyn Fn(&T) -> DomNode + 'static>,
) -> VecDeque<(T, DomNode)> {
    let pairs_top = get_pairs_top(&mut real_child, &mut new_child);
    let mut pairs_bottom = get_pairs_bottom(&mut real_child, &mut new_child);

    let last_before: DomId = find_first_dom(&pairs_bottom).unwrap_or(comment_id);
    let mut pairs_middle = get_pairs_middle(
        parent_id,
        last_before,
        real_child,
        new_child,
        get_key,
        render,
    );

    let mut pairs = pairs_top;
    pairs.append(&mut pairs_middle);
    pairs.append(&mut pairs_bottom);
    pairs
}

fn find_first_dom<T>(list: &VecDeque<(T, DomNode)>) -> Option<DomId> {
    if let Some((_, first)) = list.front() {
        return Some(first.id_dom());
    }

    None
}

// Try to match starting from top
fn get_pairs_top<T: PartialEq>(
    current: &mut VecDeque<(T, DomNode)>,
    new_child: &mut VecDeque<T>,
) -> VecDeque<(T, DomNode)> {
    let mut pairs_top = VecDeque::new();

    loop {
        let node = current.pop_front();
        let child = new_child.pop_front();

        match (node, child) {
            (Some((id1, node)), Some(id2)) => {
                if id1 == id2 {
                    pairs_top.push_back((id1, node));
                    continue;
                }

                current.push_front((id1, node));
                new_child.push_front(id2);
            }
            (Some(pair), None) => {
                current.push_front(pair);
            }
            (None, Some(child)) => {
                new_child.push_front(child);
            }
            (None, None) => {}
        }

        return pairs_top;
    }
}

// Try to match starting from bottom
fn get_pairs_bottom<T: PartialEq>(
    current: &mut VecDeque<(T, DomNode)>,
    new_child: &mut VecDeque<T>,
) -> VecDeque<(T, DomNode)> {
    let mut pairs_bottom = VecDeque::new();

    loop {
        let node = current.pop_back();
        let child = new_child.pop_back();

        match (node, child) {
            (Some((id1, node)), Some(id2)) => {
                if id1 == id2 {
                    pairs_bottom.push_front((id1, node));
                    continue;
                }

                current.push_back((id1, node));
                new_child.push_back(id2);
            }
            (Some(node), None) => {
                current.push_back(node);
            }
            (None, Some(child)) => {
                new_child.push_back(child);
            }
            (None, None) => {}
        }

        return pairs_bottom;
    }
}

fn get_pairs_middle<T: PartialEq + Clone, K: Eq + Hash>(
    parent_id: DomId,
    last_before: DomId,
    real_child: VecDeque<(T, DomNode)>,
    new_child: VecDeque<T>,
    get_key: Rc<dyn Fn(&T) -> K + 'static>,
    render: Rc<dyn Fn(&T) -> DomNode + 'static>,
) -> VecDeque<(T, DomNode)> {
    let mut pairs_middle: VecDeque<(T, DomNode)> = VecDeque::new();

    let mut real_node: CacheNode<K, T> = CacheNode::new(get_key, render);

    for (id, node) in real_child.into_iter() {
        real_node.insert(&id, node);
    }

    let mut last_before = last_before;

    for item in new_child.into_iter().rev() {
        let node = real_node.get_or_create(&item);
        let node_id = node.id_dom();
        pairs_middle.push_front((item, node));

        get_driver_dom().insert_before(parent_id, node_id, Some(last_before));
        last_before = node_id;
    }

    pairs_middle
}

struct CacheNode<K: Eq + Hash, T> {
    get_key: Rc<dyn Fn(&T) -> K + 'static>,
    create_new: Rc<dyn Fn(&T) -> DomNode + 'static>,
    data: HashMap<K, VecDeque<(T, DomNode)>>,
}

impl<K: Eq + Hash, T: PartialEq + Clone> CacheNode<K, T> {
    pub fn new(
        get_key: Rc<dyn Fn(&T) -> K + 'static>,
        create_new: Rc<dyn Fn(&T) -> DomNode + 'static>,
    ) -> CacheNode<K, T> {
        CacheNode {
            get_key,
            create_new,
            data: HashMap::new(),
        }
    }

    pub fn insert(&mut self, item: &T, element: DomNode) {
        let key = (self.get_key)(item);
        let queue = self.data.entry(key).or_default();
        queue.push_back((item.clone(), element));
    }

    pub fn get_or_create(&mut self, item: &T) -> DomNode {
        let key = (self.get_key)(item);
        let element = self.data.entry(key).or_default().pop_front();

        let CacheNode { create_new, .. } = self;

        match element {
            Some((old_item, node)) if old_item == *item => node,
            Some((_old_item, _node)) => create_new(item),
            None => create_new(item),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::reorder_nodes;
    use crate::{DomId, DomNode, computed::struct_mut::ValueMut};

    #[derive(Clone, PartialEq, Debug)]
    struct Item {
        id: u32,
        label: String,
    }

    #[test]
    fn rerenders_node_when_item_changes_but_key_stays_the_same() {
        let old_item = Item {
            id: 1,
            label: "old".to_string(),
        };
        let new_item = Item {
            id: 1,
            label: "new".to_string(),
        };

        let render_calls = Rc::new(ValueMut::new(0usize));
        let render_calls_for_closure = render_calls.clone();

        let render: Rc<dyn Fn(&Item) -> DomNode> = Rc::new(move |item| {
            render_calls_for_closure.change(|count| *count += 1);
            DomNode::from(item.label.clone())
        });

        let old_node = render(&old_item);
        render_calls.set(0);

        let result = reorder_nodes(
            DomId::from_u64(100),
            DomId::from_u64(101),
            std::collections::VecDeque::from([(old_item.clone(), old_node)]),
            std::collections::VecDeque::from([new_item]),
            Rc::new(|item: &Item| item.id),
            render,
        );

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0.label, "new");
        assert_eq!(
            render_calls.get(),
            1,
            "Expected rerender when item changes and key stays the same"
        );
    }

    #[test]
    fn reuses_cached_node_when_item_value_is_unchanged() {
        let item = Item {
            id: 1,
            label: "same".to_string(),
        };

        let render_calls = Rc::new(ValueMut::new(0usize));
        let render_calls_for_closure = render_calls.clone();

        let render: Rc<dyn Fn(&Item) -> DomNode> = Rc::new(move |item| {
            render_calls_for_closure.change(|count| *count += 1);
            DomNode::from(item.label.clone())
        });

        let old_node = render(&item);
        let old_node_id = old_node.id_dom();
        render_calls.set(0);

        let result = reorder_nodes(
            DomId::from_u64(200),
            DomId::from_u64(201),
            std::collections::VecDeque::from([(item.clone(), old_node)]),
            std::collections::VecDeque::from([item]),
            Rc::new(|item: &Item| item.id),
            render,
        );

        assert_eq!(result.len(), 1);
        assert_eq!(render_calls.get(), 0, "Expected cached DomNode reuse");
        assert_eq!(
            result[0].1.id_dom(),
            old_node_id,
            "Expected to get the same DomNode from cache"
        );
    }
}
