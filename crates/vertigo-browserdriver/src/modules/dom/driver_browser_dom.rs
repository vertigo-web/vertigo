use wasm_bindgen::prelude::Closure;
use std::rc::Rc;

use vertigo::{EventCallback, KeyDownEvent, NodeRefsItem, RealDomId, computed::Dependencies};

use super::js_dom::DriverBrowserDomJs;
use super::driver_data::DriverData;
use super::element_wrapper::{DomElement, DomText};
use super::visited_node_manager::VisitedNodeManager;

type KeydownClosureType = Closure<dyn Fn(Option<u64>, String, String, bool, bool, bool, bool) -> bool>;

pub struct DriverDomInner {
    data: Rc<DriverData>,
    dom_js: Rc<DriverBrowserDomJs>,
    _mouse_down: Closure<dyn Fn(u64)>,
    _mouse_enter: Closure<dyn Fn(Option<u64>)>,
    _keydown: KeydownClosureType,
    _oninput: Closure<dyn Fn(u64, String)>,
}

pub struct DriverBrowserDom {
    inner: Rc<DriverDomInner>,
}

impl DriverBrowserDom {
    pub fn new(dependencies: &Dependencies) -> DriverBrowserDom {
        let data = DriverData::new();

        let mouse_down = {
            let data = data.clone();

            Closure::new(Box::new(move |dom_id: u64| {
                let event_to_run = data.find_event_click(RealDomId::from_u64(dom_id));

                if let Some(event_to_run) = event_to_run {
                    event_to_run();
                }
            }))
        };

        let mouse_enter: Closure<dyn Fn(Option<u64>)> = {
            let data = data.clone();
            let current_visited = VisitedNodeManager::new(&data, dependencies);

            Closure::new(Box::new(move |dom_id: Option<u64>| {
                match dom_id {
                    Some(dom_id) => {
                        let nodes = data.find_all_nodes(RealDomId::from_u64(dom_id));
                        current_visited.push_new_nodes(nodes);
                    },
                    None => {
                        current_visited.clear();
                    }
                }
            }))
        };

        let keydown: KeydownClosureType = {
            let data = data.clone();

            Closure::new(Box::new(move |
                dom_id: Option<u64>,
                key: String,
                code: String,
                alt_key: bool,
                ctrl_key: bool,
                shift_key: bool,
                meta_key: bool
            | -> bool {
                let event = KeyDownEvent {
                    key,
                    code,
                    alt_key,
                    ctrl_key,
                    shift_key,
                    meta_key,
                };

                let id = match dom_id {
                    Some(id) => RealDomId::from_u64(id),
                    None => RealDomId::root()
                };

                let event_to_run = data.find_event_keydown(id);

                if let Some(event_to_run) = event_to_run {
                    let prevent_default = event_to_run(event);

                    if prevent_default {
                        return true;
                    }
                }

                false
            }))
        };

        let oninput: Closure<dyn Fn(u64, String)> = {
            let data = data.clone();

            Closure::new(Box::new(move |dom_id: u64, text: String| {
                let event_to_run = data.find_event_on_input(RealDomId::from_u64(dom_id));

                if let Some(event_to_run) = event_to_run {
                    event_to_run(text);
                }
            }))
        };

        let dom_js = Rc::new(DriverBrowserDomJs::new(
            &mouse_down,
            &mouse_enter,
            &keydown,
            &oninput
        ));

        let root_id = RealDomId::root();
        data.elements.insert(root_id.clone(), DomElement::new(dom_js.clone(), root_id.clone(), "div"));
        dom_js.mount_root(root_id.to_u64());

        DriverBrowserDom {
            inner: Rc::new(DriverDomInner {
                data,
                dom_js,
                _mouse_down: mouse_down,
                _mouse_enter: mouse_enter,
                _keydown: keydown,
                _oninput: oninput,
            })
        }
    }
}

impl DriverBrowserDom {
    pub fn create_node(&self, id: RealDomId, name: &'static str) {
        let element = DomElement::new(self.inner.dom_js.clone(), id.clone(), name);
        self.inner.data.elements.insert(id, element);
    }

    pub fn create_text(&self, id: RealDomId, value: &str) {
        let text = DomText::new(self.inner.dom_js.clone(), id.clone(), value);
        self.inner.data.texts.insert(id, text);
    }

    pub fn get_ref(&self, id: RealDomId) -> Option<NodeRefsItem> {
        self.inner.data.elements.must_get(&id, |elem| {
            elem.to_ref()
        })
    }

    pub fn update_text(&self, id: RealDomId, value: &str) {
        self.inner.data.texts.must_get(&id, move |elem| {
            elem.update_text(value);
        });
    }

    pub fn set_attr(&self, id: RealDomId, key: &'static str, value: &str) {
        self.inner.data.elements.must_get(&id, move |elem| {
            elem.set_attr(key, value);
        });
    }

    pub fn remove_attr(&self, id: RealDomId, name: &'static str) {
        self.inner.data.elements.must_get(&id, move |elem| {
            elem.remove_attr(name);
        });
    }

    pub fn remove_text(&self, id: RealDomId) {
        let status1 = self.inner.data.child_parent.remove(&id);
        if status1.is_none() {
            log::error!("remove text -> child_parent -> missing id={}", id);
        }

        let status2 = self.inner.data.texts.remove(&id);
        if status2.is_none() {
            log::error!("remove text -> texts -> missing id={}", id);
        }
    }

    pub fn remove_node(&self, id: RealDomId) {
        let status1 = self.inner.data.child_parent.remove(&id);
        if status1.is_none() {
            log::error!("remove node -> child_parent -> missing id={}", id);
        }

        let status2 = self.inner.data.elements.remove(&id);
        if status2.is_none() {
            log::error!("remove node -> elements -> missing id={}", id);
        }
    }

    pub fn insert_before(&self, parent: RealDomId, child: RealDomId, ref_id: Option<RealDomId>) {

        let is_insert = self.inner.data.elements.try_get(&parent, {
            let child = child.clone();
            let ref_id = ref_id.clone();

            move |node| {
                node.insert_before(child, ref_id);
                true
            }
        });

        if is_insert == Some(true) {
            self.inner.data.child_parent.insert(child, parent);
            return;
        }


        let is_insert = self.inner.data.texts.try_get(&parent, |node| {
            node.insert_before(child.clone(), ref_id);
            true
        });

        if is_insert == Some(true) {
            self.inner.data.child_parent.insert(child, parent);
            return;
        }

        log::error!("insert_before -> Missing element with id={}", parent);
    }

    pub fn insert_css(&self, selector: &str, value: &str) {
        self.inner.dom_js.insert_css(selector, value);
    }

    pub fn set_event(&self, id: RealDomId, callback: EventCallback) {
        self.inner.data.elements.must_change(&id, move |node| {
            match callback {
                EventCallback::OnClick { callback } => {
                    node.on_click = callback;
                },
                EventCallback::OnInput { callback } => {
                    node.on_input = callback;
                },
                EventCallback::OnMouseEnter { callback } => {
                    node.on_mouse_enter = callback;
                },
                EventCallback::OnMouseLeave { callback } => {
                    node.on_mouse_leave = callback;
                },
                EventCallback::OnKeyDown { callback} => {
                    node.on_keydown = callback;
                }
            }
        });
    }
}
