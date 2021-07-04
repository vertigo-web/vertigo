
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    rc::Rc,
};
use crate::element_wrapper::ElementWrapper;
use wasm_bindgen::{JsCast, prelude::Closure, JsValue};
use web_sys::{Document, Event, HtmlHeadElement, Node, HtmlInputElement, HtmlTextAreaElement, Window, Element};

use vertigo::{DomDriver, DomDriverTrait, EventCallback, FetchMethod, HashRoutingReceiver, NodeRefsItem, RealDomId, computed::Dependencies, utils::{
        BoxRefCell,
        DropResource,
    }};

use crate::dom_event::{DomEvent};
use crate::fetch;
use crate::dom_utils;
use crate::{
    element_wrapper::ElementItem,
    events::{
        input::create_input_event,
        keydown::create_keydown_event,
        mousedown::create_mousedown_event,
        mouseenter::create_mouseenter_event
    }
};

struct DomDriverBrowserInner {
    window: Window,
    document: Document,
    head: HtmlHeadElement,
    root: Element,
    elements: HashMap<RealDomId, ElementWrapper>,
    child_parent: HashMap<RealDomId, RealDomId>,            //child -> parent
    _dom_event_disconnect: Vec<DomEvent>,
}

impl DomDriverBrowserInner {
    fn new() -> Rc<BoxRefCell<Self>> {
        let (window, document, head) = dom_utils::get_window_elements();

        let root_id = RealDomId::root();
        let root = dom_utils::create_root(&document, &root_id);
        let mut elements = HashMap::new();
        elements.insert(root_id, ElementWrapper::from_node(root.clone()));

        let dom_driver_inner = DomDriverBrowserInner {
            window,
            document,
            head,
            root,
            elements,
            child_parent: HashMap::new(),
            _dom_event_disconnect: Vec::new(),
        };

        Rc::new(BoxRefCell::new(
            dom_driver_inner,
            "DomDriverBrowserInner"
        ))
    }

    fn create_node(&mut self, id: RealDomId, name: &'static str) {
        let node = dom_utils::create_node(&self.document, &id, name);
        self.elements.insert(id, ElementWrapper::from_node(node));
    }

    fn create_text(&mut self, id: RealDomId, value: &str) {
        let text = self.document.create_text_node(value);
        self.elements.insert(id, ElementWrapper::from_text(text));
    }

    fn get_ref(&self, id: RealDomId) -> Option<NodeRefsItem> {
        if let Some(elem) = self.elements.get(&id) {
            return elem.to_ref();
        }

        None
    }

    fn update_text(&mut self, id: RealDomId, value: &str) {
        let elem = self.elements.get(&id);

        if let Some(elem) = elem {
            match elem {
                ElementWrapper { item: ElementItem::Element { .. }, ..} => {
                    log::error!("Cannot update text on node id={}", id);
                },
                ElementWrapper { item: ElementItem::Text { text }, ..} => {
                    text.set_data(value);
                }
            }
            return;
        }

        log::error!("Missing element with id={}", id);
    }

    fn set_attr(&mut self, id: RealDomId, name: &'static str, value: &str) {
        let elem = self.elements.get_mut(&id);

        if let Some(elem) = elem {
            match elem {
                ElementWrapper { item: ElementItem::Element { node }, ..} => {
                    node.set_attribute(name, value).unwrap();

                    if name == "value" {
                        let input_node = node.clone().dyn_into::<HtmlInputElement>();

                        if let Ok(input_node) = input_node {
                            input_node.set_value(value);
                            return;
                        }

                        let textarea_node = node.clone().dyn_into::<HtmlTextAreaElement>();

                        if let Ok(textarea_node) = textarea_node {
                            textarea_node.set_value(value);
                            return;
                        }
                    }
                },
                ElementWrapper { item: ElementItem::Text { .. }, ..} => {
                    log::error!("Cannot set attribute on a text node id={}", id);
                }
            }
            return;
        }

        log::error!("Missing element with id={}", id);
    }

    fn remove_attr(&mut self, id: RealDomId, name: &'static str) {
        let elem = self.elements.get_mut(&id);

        if let Some(elem) = elem {
            match elem {
                ElementWrapper { item: ElementItem::Element { node }, ..} => {
                    node.remove_attribute(name).unwrap();
                },
                ElementWrapper { item: ElementItem::Text { .. }, ..} => {
                    log::error!("Cannot remove attribute on a text node id={}", id);
                }
            }
            return;
        }

        log::error!("Missing element with id={}", id);
    }

    fn remove(&mut self, id: RealDomId) {
        let elem = self.elements.remove(&id);

        if let Some(elem) = elem {
            match elem {
                ElementWrapper { item: ElementItem::Element { node }, ..} => {
                    node.remove();
                },
                ElementWrapper { item: ElementItem::Text { text }, ..} => {
                    text.remove();
                }
            }
            return;
        }

        self.child_parent.remove(&id);

        log::error!("Missing element with id={}", id);
    }

    fn get_node(&self, ref_id: &RealDomId) -> Option<Node> {
        let child_item = self.elements.get(ref_id);

        match child_item {
            Some(ElementWrapper { item: ElementItem::Element { node }, ..}) => {
                let node = node.clone().dyn_into::<Node>().unwrap();
                Some(node)
            },
            Some(ElementWrapper { item: ElementItem::Text { text }, ..}) => {
                let node = text.clone().dyn_into::<Node>().unwrap();
                Some(node)
            },
            None => {
                log::error!("no element was found id={}", ref_id);
                None
            }
        }
    }

    fn insert_before(&mut self, parent: RealDomId, child: RealDomId, ref_id: Option<RealDomId>) {
        let parent_item: Node = self.get_node(&parent).unwrap();
        let child_item = self.get_node(&child).unwrap();

        match ref_id {
            Some(ref_id) => {
                let rel_node = self.get_node(&ref_id).unwrap();

                parent_item.insert_before(&child_item, Some(&rel_node)).unwrap();
            },
            None => {
                parent_item.insert_before(&child_item, None).unwrap();
            },
        }

        self.child_parent.insert(child, parent);
    }

    fn set_event(&mut self, node_id: RealDomId, callback: EventCallback) {
        let item = self.elements.get_mut(&node_id).unwrap();

        match callback {
            EventCallback::OnClick { callback } => {
                item.on_click = callback;
            },
            EventCallback::OnInput { callback } => {
                item.on_input = callback;
            },
            EventCallback::OnMouseEnter { callback } => {
                item.on_mouse_enter = callback;
            },
            EventCallback::OnMouseLeave { callback } => {
                item.on_mouse_leave = callback;
            },
            EventCallback::OnKeyDown { callback} => {
                item.on_keydown = callback;
            }
        }
    }

    fn insert_css(&self, selector: &str, value: &str) {
        let style = self.document.create_element("style").unwrap();
        let content = self.document.create_text_node(format!("{} {{ {} }}", selector, value).as_str());
        style.append_child(&content).unwrap();

        self.head.append_child(&style).unwrap();
    }

    fn get_hash_location(&self) -> String {
        self.window.location().hash().expect("Can't read hash from location bar")
    }

    fn push_hash_location(&self, path: &str) {
        let path = format!("#{}", path);
        let history = self.window.history().expect("Can't read history from window");
        history.push_state_with_url(&JsValue::from_str(""), "", Some(&path)).expect("Can't push state to history");
    }
}


#[derive(Clone)]
pub struct DomDriverBrowser {
    driver: Rc<BoxRefCell<DomDriverBrowserInner>>,
}

impl DomDriverBrowser {
    pub fn new(dependencies: &Dependencies) -> DomDriver {
        let driver = DomDriverBrowserInner::new();

        let dom_driver_browser = DomDriverBrowser {
            driver,
        };

        let root = dom_driver_browser.driver.get(|state| {
            state.root.clone()
        });

        let document = dom_driver_browser.get_document();

        let dom_event_disconnect = vec![
            create_mousedown_event(&dom_driver_browser, &root),
            create_input_event(&dom_driver_browser, &root),
            create_mouseenter_event(&dom_driver_browser, &root, dependencies),
            create_keydown_event(&document, &dom_driver_browser),
        ];

        dom_driver_browser.driver.change(dom_event_disconnect, |state, dom_event_disconnect| {
            state._dom_event_disconnect = dom_event_disconnect;
        });

        DomDriver::new(dom_driver_browser, Box::new(|fut: Pin<Box<dyn Future<Output = ()> + 'static>>| {
            wasm_bindgen_futures::spawn_local(fut);
        }))
    }

    fn get_document(&self) -> Document {
        self.driver.get(|state| {
            state.document.clone()
        })
    }

    pub fn get_from_node<R>(self: &DomDriverBrowser, node_id: &RealDomId, map: fn(&ElementWrapper) -> Option<R>) -> Option<R> {
        self.driver.get_with_context((node_id, map), |state, (node_id, map)| {
            match state.elements.get(node_id) {
                Some(element) => map(element),
                None => {
                    log::error!("get_from_node - missing node {}", node_id);
                    None
                }
            }
        })
    }


    pub fn find_all_nodes(
        self: &DomDriverBrowser,
        id: RealDomId,
    ) -> Vec<RealDomId> {
        self.driver.get_with_context(
            id,
            |state, id| -> Vec<RealDomId> {
                if id == RealDomId::root() {
                    return vec![RealDomId::root()];
                }
                
                let mut wsk = id.clone();
                let mut count = 0;
                let mut out: Vec<RealDomId> = Vec::new();

                loop {
                    out.push(wsk.clone());

                    count += 1;

                    if count > 100 {
                        log::error!("Too many nested levels");
                        return out;
                    }

                    let parent = state.child_parent.get(&wsk);
                    if let Some(parent) = parent {
                        if *parent == RealDomId::root() {
                            out.push(parent.clone());
                            return out;
                        } else {
                            wsk = parent.clone();
                        }
                    } else {
                        log::error!("It should never have happened {:?}", id);
                        return out;
                    }
                }
            }
        )
    }

    pub fn find_dom_id(self: &DomDriverBrowser, event: &web_sys::Event) -> RealDomId {
        let target = event.target().unwrap();
        let element = target.dyn_ref::<Element>().unwrap();
    
        let option_id: Option<String> = (*element).get_attribute("data-id");
        let option_id = match option_id {
            Some(option_id) => option_id,
            None => {
                return RealDomId::root();
            }
        };
    
        let id = option_id.parse::<u64>();
    
        match id {
            Ok(id) => RealDomId::from_u64(id),
            Err(_) => RealDomId::root()
        }
    }
}

impl DomDriverTrait for DomDriverBrowser {
    fn create_node(&self, id: RealDomId, name: &'static str) {
        self.driver.change((id, name), |state, (id, name)| {
            state.create_node(id, name);
        });
    }

    fn create_text(&self, id: RealDomId, value: &str) {
        self.driver.change((id, value), |state, (id, value)| {
            state.create_text(id, value);
        });
    }

    fn get_ref(&self, id: RealDomId) -> Option<NodeRefsItem> {
        self.driver.change(id, |state, id| {
            state.get_ref(id)
        })
    }

    fn update_text(&self, id: RealDomId, value: &str) {
        self.driver.change((id, value), |state, (id, value)| {
            state.update_text(id, value);
        });
    }

    fn set_attr(&self, id: RealDomId, key: &'static str, value: &str) {
        self.driver.change((id, key, value), |state, (id, key, value)| {
            state.set_attr(id, key, value);
        });
    }

    fn remove_attr(&self, id: RealDomId, name: &'static str) {
        self.driver.change((id, name), |state, (id, name)| {
            state.remove_attr(id, name);
        });
    }

    fn remove(&self, id: RealDomId) {
        self.driver.change(id, |state, id| {
            state.remove(id);
        });
    }

    fn insert_before(&self, parent: RealDomId, child: RealDomId, ref_id: Option<RealDomId>) {
        self.driver.change((parent, ref_id, child), |state, (parent, ref_id, child)| {
            state.insert_before(parent, child, ref_id);
        });
    }

    fn insert_css(&self, class: &str, value: &str) {
        self.driver.change((class, value), |state, (class, value)| {
            state.insert_css(class, value);
        });
    }

    fn set_event(&self, node: RealDomId, callback: EventCallback) {
        self.driver.change((node, callback), |state, (node, callback)| {
            state.set_event(node, callback);
        });
    }

    fn fetch(&self, method: FetchMethod, url: String, headers: Option<HashMap<String, String>>, body: Option<String>) -> Pin<Box<dyn Future<Output=Result<String, String>> + 'static>> {
        fetch::fetch(method, url, headers, body)
    }

    fn get_hash_location(&self) -> String {
        self.driver.get(|state| {
            let mut path = state.get_hash_location();

            // Remove '#'
            match path.char_indices().nth(1) {
                Some((pos, _)) => {
                    path.drain(..pos);
                }
                None => {
                    path.clear();
                }
            }

            path
        })
    }

    fn push_hash_location(&self, path: &str) {
        self.driver.change(path, |state, path| {
            state.push_hash_location(path);
        });
    }

    fn on_hash_route_change(&self, on_change: Box<dyn Fn(String)>) -> HashRoutingReceiver {
        let myself = self.clone();

        let on_popstate = Closure::<dyn Fn(Event)>::new({
            move |_: Event| {
                let path = myself.get_hash_location();
                on_change(path);
            }
        });

        self.driver.change(on_popstate, |state, on_popstate| {
            state.window.set_onpopstate(Some(on_popstate.as_ref().unchecked_ref()));
            on_popstate.forget();
        });

        HashRoutingReceiver::new(self.clone())
    }

    fn clear_hash_route_callback(&self) {
        self.driver.change((), |state, _| {
            state.window.set_onpopstate(None);
        });
    }

    fn set_interval(&self, time: u32, func: Box<dyn Fn()>) -> DropResource {
        let drop_resource = crate::set_interval::Interval::new(time, func);

        DropResource::new(move || {
            drop_resource.off();
        })
    }
}
