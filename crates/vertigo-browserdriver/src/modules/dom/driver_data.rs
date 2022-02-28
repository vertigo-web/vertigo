use std::hash::Hash;
use std::rc::Rc;
use vertigo::{dev::{RealDomId, EventCallback}, KeyDownEvent};
use std::fmt::Display;
use vertigo::struct_mut::HashMapMut;

use super::element_wrapper::DomElement;

struct HashMapRcWithLabel<K: Eq + Hash + Display, V> {
    label: &'static str,
    data: HashMapMut<K, V>
}

impl<K: Eq + Hash + Display, V> HashMapRcWithLabel<K, V> {
    pub fn new(label: &'static str) -> HashMapRcWithLabel<K, V> {
        HashMapRcWithLabel {
            label,
            data: HashMapMut::new(),
        }
    }

    pub fn insert(&self, key: K, value: V) -> Option<V> {
        self.data.insert(key, value)
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        self.data.remove(key)
    }

    pub fn must_get<R, F: FnOnce(&V) -> R>(&self, key: &K, callback: F) -> Option<R> {
        let state = self.data.get_and_map(key, callback);

        if state.is_none() {
            log::error!("{} -> get -> Missing element with id={}", self.label, key);
        }

        state
    }

    pub fn must_change<R, F: FnOnce(&mut V) -> R>(&self, key: &K, callback: F) -> Option<R> {
        let item = self.data.must_change(key, callback);

        if item.is_none() {
            let label = self.label;
            log::error!("{label} ->change ->  Missing element with id={key}");
        }

        item
    }

    pub fn filter_and_map<R>(&self, map: fn(&V) -> Option<R>) -> Vec<R> {
        self.data.filter_and_map(map)
    }
}

pub struct DriverData {
    elements: HashMapRcWithLabel<RealDomId, DomElement>,
    child_parent: HashMapRcWithLabel<RealDomId, RealDomId>, // child -> parent
}

impl DriverData {
    pub fn new() -> Rc<DriverData> {
        Rc::new(DriverData {
            elements: HashMapRcWithLabel::new("DriverData elements"),
            child_parent: HashMapRcWithLabel::new("DriverData child_parent"),
        })
    }

    pub fn create_node(&self, id: RealDomId) {
        self.elements.insert(id, DomElement::new());
    }

    pub fn remove_text(&self, id: RealDomId) {
        self.child_parent.remove(&id);
    }

    pub fn remove_node(&self, id: RealDomId) {
        self.child_parent.remove(&id);
        self.elements.remove(&id);
    }

    pub fn set_parent(&self, child: RealDomId, parent: RealDomId) {
        self.child_parent.insert(child, parent);
    }

    pub fn set_event(&self, id: RealDomId, callback: EventCallback) {
        self.elements.must_change(&id, move |node|
            match callback {
                EventCallback::OnClick { callback } => {
                    node.on_click = callback;
                }
                EventCallback::OnInput { callback } => {
                    node.on_input = callback;
                }
                EventCallback::OnMouseEnter { callback } => {
                    node.on_mouse_enter = callback;
                }
                EventCallback::OnMouseLeave { callback } => {
                    node.on_mouse_leave = callback;
                }
                EventCallback::OnKeyDown { callback } => {
                    node.on_keydown = callback;
                },
                EventCallback::HookKeyDown { callback } => {
                    node.hook_keydown = callback;
                }
            }
        );
    }

    pub fn find_all_nodes(&self, id: RealDomId) -> Vec<RealDomId> {
        if id == RealDomId::root() {
            return vec![RealDomId::root()];
        }

        let mut wsk = id;
        let mut count = 0;
        let mut out: Vec<RealDomId> = vec![wsk];

        loop {
            count += 1;

            if count > 100 {
                log::error!("Too many nested levels");
                return out;
            }

            let parent = self.child_parent.must_get(&wsk, |item| *item);

            if let Some(parent) = parent {
                out.push(parent);

                if parent == RealDomId::root() {
                    return out;
                } else {
                    wsk = parent;
                }
            } else {
                log::error!("It should never have happened {:?}", id);
                return out;
            }
        }
    }

    pub fn get_from_node<R>(&self, node_id: &RealDomId, map: fn(&DomElement) -> Option<R>) -> Option<R> {
        self.elements.must_get(node_id, map).flatten()
    }

    pub fn find_event_click(&self, id: RealDomId) -> Option<Rc<dyn Fn()>> {
        let all_nodes = self.find_all_nodes(id);

        for node_id in all_nodes {

            let on_click = self.get_from_node(
                &node_id,
                |elem| elem.on_click.clone()
            );

            if on_click.is_some() {
                return on_click;
            }
        }

        None
    }

    pub fn find_hook_keydown(&self) -> Vec<Rc<dyn Fn(KeyDownEvent) -> bool>> {
        self.elements.filter_and_map(|item| -> Option<Rc<dyn Fn(KeyDownEvent) -> bool>> {
            item.hook_keydown.clone()
        })
    }

    pub fn find_event_keydown(&self, id: RealDomId) -> Option<Rc<dyn Fn(KeyDownEvent) -> bool>> {
        let all_nodes = self.find_all_nodes(id);

        for node_id in all_nodes {
            let on_key = self.get_from_node(
                &node_id,
                |elem| elem.on_keydown.clone()
            );

            if on_key.is_some() {
                return on_key;
            }
        }

        None
    }

    pub fn find_event_on_input(&self, id: RealDomId) -> Option<Rc<dyn Fn(String)>> {
        self.get_from_node(
            &id,
            |elem| elem.on_input.clone()
        )
    }
}
