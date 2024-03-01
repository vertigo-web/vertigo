use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::rc::Rc;

use crate::dom::dom_id::DomId;
use crate::struct_mut::ValueMut;
use crate::{get_driver, Computed, DomComment, DomNode};

//TODO - Check out other options for reading refs

// pub struct ListRendered<T: Clone> {
//     comment: DomFragment,
//     pub refs: Computed<Vec<(T, DomElementRef)>>,
// }

// impl<T: Clone> EmbedDom for ListRendered<T> {
//     fn embed(self) -> DomNodeFragment {
//         self.comment.into()
//     }
// }

// impl<T: Clone> From<ListRendered<T>> for DomNodeFragment {
//     fn from(val: ListRendered<T>) -> Self {
//         val.comment.into()
//     }
// }

// fn get_refs<T: Clone>(list: &VecDeque<(T, DomElement)>) -> Vec<(T, DomElementRef)> {
//     let mut result = Vec::new();

//     for (item, element) in list {
//         result.push((item.clone(), element.get_ref()));
//     }

//     result
// }

pub fn render_list<T: PartialEq + Clone + 'static, K: Eq + Hash>(
    computed: Computed<Vec<T>>,
    get_key: impl Fn(&T) -> K + 'static,
    render: impl Fn(&T) -> DomNode + 'static,
) -> DomNode {
    let get_key = Rc::new(get_key);
    let render = Rc::new(render);

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

fn reorder_nodes<T: PartialEq, K: Eq + Hash>(
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

fn get_pairs_middle<T: PartialEq, K: Eq + Hash>(
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

    let driver = get_driver();
    let mut last_before = last_before;

    for item in new_child.into_iter().rev() {
        let node = real_node.get_or_create(&item);
        let node_id = node.id_dom();
        pairs_middle.push_front((item, node));

        driver
            .inner
            .dom
            .insert_before(parent_id, node_id, Some(last_before));
        last_before = node_id;
    }

    pairs_middle
}

struct CacheNode<K: Eq + Hash, T> {
    get_key: Rc<dyn Fn(&T) -> K + 'static>,
    create_new: Rc<dyn Fn(&T) -> DomNode + 'static>,
    data: HashMap<K, VecDeque<DomNode>>,
}

impl<K: Eq + Hash, T> CacheNode<K, T> {
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
        let item = self.data.entry(key).or_default();
        item.push_back(element);
    }

    pub fn get_or_create(&mut self, item: &T) -> DomNode {
        let key = (self.get_key)(item);
        let element = self.data.entry(key).or_default().pop_front();

        let CacheNode { create_new, .. } = self;

        match element {
            Some(node) => node,
            None => create_new(item),
        }
    }
}
