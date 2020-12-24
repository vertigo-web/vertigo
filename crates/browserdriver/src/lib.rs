use web_sys::{Document, Element, Text, HtmlHeadElement, Node, HtmlInputElement, HtmlTextAreaElement};
use std::rc::Rc;
use std::collections::HashMap;

use vertigo::{DomDriver, computed::BoxRefCell};
use vertigo::{DomDriverTrait, FetchMethod, FetchError};
use vertigo::RealDomId;
use vertigo::EventCallback;

use wasm_bindgen::JsCast;
use dom_event::{DomEventDisconnect, DomEvent, /*DomEventKeyboard, DomEventMouse */};

use std::pin::Pin;
use std::future::Future;

mod dom_event;
mod fetch;

fn get_document() -> (Document, HtmlHeadElement) {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let head = document.head().unwrap();
    (document, head)
}

fn create_node(document: &Document, id: &RealDomId, name: &'static str) -> Element {
    let node = document.create_element(name).unwrap();
    let id_str = format!("{}", id.to_u64());
    node.set_attribute("data-id", id_str.as_str()).unwrap();
    node
}

fn create_root(document: &Document, root_id: &RealDomId) -> Element {
    let body = document.body().expect("document should have a body");
    let root = create_node(document, root_id, "div");
    body.append_child(&root).unwrap();
    root
}

enum ElementItem {
    Element {
        node: Element,
    },
    Text {
        text: Text
    },
}

impl ElementItem {
    pub fn from_node(node: Element) -> ElementItem {
        ElementItem::Element { node }
    }

    pub fn from_text(text: Text) -> ElementItem {
        ElementItem::Text { text }
    }
}

struct ElementWrapper {
    item: ElementItem,
    on_click: Option<Rc<dyn Fn()>>,
    on_input: Option<Rc<dyn Fn(String)>>,
    on_mouse_enter: Option<Rc<dyn Fn()>>,
    on_mouse_leave: Option<Rc<dyn Fn()>>,
}

impl ElementWrapper {
    pub fn from_node(node: Element) -> ElementWrapper {
        ElementWrapper {
            item: ElementItem::from_node(node),
            on_click: None,
            on_input: None,
            on_mouse_enter: None,
            on_mouse_leave: None,
        }
    }

    pub fn from_text(text: Text) -> ElementWrapper {
        ElementWrapper {
            item: ElementItem::from_text(text),
            on_click: None,
            on_input: None,
            on_mouse_enter: None,
            on_mouse_leave: None,
        }
    }
}

fn find_event<T: Clone>(
    inner: &Rc<BoxRefCell<DomDriverBrowserInner>>,
    id: u64,
    find_event_on_click_item: fn(&ElementWrapper) -> &Option<T>,
) -> Option<T> {
    let id = RealDomId::from_u64(id);

    let on_click = inner.get_with_context(
        (id, find_event_on_click_item),
        |state, (id, find_event_on_click_item)| -> Option<T> {
            let mut wsk = id;
            let mut count = 0;

            loop {
                count += 1;

                if count > 100 {
                    log::error!("Too many nested levels");
                    return None;
                }

                let item = state.elements.get(&wsk).unwrap();

                let item_inner = find_event_on_click_item(item);

                if let Some(on_click) = item_inner {
                    return Some(on_click.clone());
                }

                let parent = state.child_parent.get(&wsk);
                if let Some(parent) = parent {
                    wsk = parent.clone();
                } else {
                    return None;
                }
            }
        }
    );

    on_click
}

fn find_event_on_input(inner: &Rc<BoxRefCell<DomDriverBrowserInner>>, id: u64) -> Option<Rc<dyn Fn(String)>> {
    let id = RealDomId::from_u64(id);

    let on_input = inner.get_with_context(
        id,
        |state, id| -> Option<Rc<dyn Fn(String)>> {
            let item = state.elements.get(&id).unwrap();

            if let Some(on_input) = &item.on_input {
                return Some(on_input.clone());
            }
            return None;
        }
    );
    on_input
}

fn find_dom_id(event: &web_sys::Event) -> u64 {
    let target = event.target().unwrap();
    let element = target.dyn_ref::<Element>().unwrap();

    let option_id: Option<String> = (*element).get_attribute("data-id");
    let id: u64 = option_id.unwrap().parse::<u64>().unwrap();
    id
}

pub struct DomDriverBrowserInner {
    document: Document,
    head: HtmlHeadElement,
    elements: HashMap<RealDomId, ElementWrapper>,
    child_parent: HashMap<RealDomId, RealDomId>,            //child -> parent
    _dom_event_disconnect: Vec<DomEventDisconnect>,
}

impl DomDriverBrowserInner {
    fn new() -> Rc<BoxRefCell<Self>> {
        let (document, head) = get_document();

        let root_id = RealDomId::root();
        let root = create_root(&document, &root_id);

        let inner = Rc::new(
            BoxRefCell::new(
                DomDriverBrowserInner {
                    document,
                    head,
                    elements: HashMap::new(),
                    child_parent: HashMap::new(),
                    _dom_event_disconnect: Vec::new(),
                }
            )
        );

        let mut dom_event_disconnect = Vec::new();

        dom_event_disconnect.push({
            let inner = inner.clone();

            DomEvent::new_event(&root, "mousedown",move |event: web_sys::Event| {
                // log::info!("event click ... {:?}", event);

                let dom_id = find_dom_id(&event);

                let event_to_run = find_event(&inner, dom_id, |item| &item.on_click);

                if let Some(event_to_run) = event_to_run {
                    event_to_run();
                }
            })
        });

        // dom_event_disconnect.push({
        //     let inner = inner.clone();

        //     ///*"mouseenter"*/
        //     DomEvent::new_event(&root, "mouseover",move |event: web_sys::Event| {
        //         log::info!("event mouseenter ... {:?}", event);

        //         let dom_id = find_dom_id(&event);

        //         let event_to_run = find_event(&inner, dom_id, |item| &item.onMouseEnter);

        //         if let Some(event_to_run) = event_to_run {
        //             event_to_run();
        //         }
        //     })
        // });

        // dom_event_disconnect.push({
        //     let inner = inner.clone();

        //     /*"mouseleave"*/
        //     DomEvent::new_event(&root, "mouseout",move |event: web_sys::Event| {
        //         log::info!("event mouseleave ... {:?}", event);

        //         let dom_id = find_dom_id(&event);

        //         let event_to_run = find_event(&inner, dom_id, |item| &item.onMouseLeave);

        //         if let Some(event_to_run) = event_to_run {
        //             event_to_run();
        //         }
        //     })
        // });


        dom_event_disconnect.push({
            let inner = inner.clone();

            DomEvent::new_event(&root, "input", move |event: web_sys::Event| {

                let dom_id = find_dom_id(&event);
                let event_to_run = find_event_on_input(&inner, dom_id);

                if let Some(event_to_run) = event_to_run {
                    let target = event.target().unwrap();
                    let input = target.dyn_ref::<HtmlInputElement>();

                    if let Some(input) = input {
                        event_to_run(input.value());
                        return;
                    }

                    let input = target.dyn_ref::<HtmlTextAreaElement>();

                    if let Some(input) = input {
                        event_to_run(input.value());
                        return;
                    }
                }
            })
        });

        inner.change(
            (dom_event_disconnect, root_id, root),
            |state, (dom_event_disconnect, root_id, root)| {
                state.elements.insert(root_id, ElementWrapper::from_node(root));
                state._dom_event_disconnect = dom_event_disconnect;
            }
        );

        inner
    }

    fn create_node(&mut self, id: RealDomId, name: &'static str) {
        let node = create_node(&self.document, &id, name);
        self.elements.insert(id, ElementWrapper::from_node(node));
    }

    fn create_text(&mut self, id: RealDomId, value: &str) {
        let text = self.document.create_text_node(value);
        self.elements.insert(id, ElementWrapper::from_text(text));
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

    fn copy_parent_from_rel(&mut self, child: &RealDomId, rel: &RealDomId) {
        let rel_parent = self.child_parent.get(rel).unwrap().clone();
        self.child_parent.insert(child.clone(), rel_parent);
    }

    fn insert_as_first_child(&mut self, parent: RealDomId, child: RealDomId) {
        let parent_item = self.get_node(&parent).unwrap();
        let child_item = self.get_node(&child).unwrap();

        parent_item.insert_before(&child_item, None).unwrap();
        self.child_parent.insert(child, parent);
    }

    fn insert_before(&mut self, ref_id: RealDomId, child: RealDomId) {
        let ref_id_item = self.get_node(&ref_id).unwrap();
        let child_item = self.get_node(&child).unwrap();

        let parent: Node = ref_id_item.parent_node().unwrap();

        parent.insert_before(&child_item, Some(&ref_id_item)).unwrap();
        self.copy_parent_from_rel(&child, &ref_id);
    }

    fn insert_after(&mut self, ref_id: RealDomId, child: RealDomId) {
        let ref_id_item = self.get_node(&ref_id).unwrap();
        let child_item = self.get_node(&child).unwrap();

        let parent: Node = ref_id_item.parent_node().unwrap();
        let next: Option<Node> = ref_id_item.next_sibling();

        match next {
            Some(next) => {
                parent.insert_before(&child_item, Some(&next)).unwrap();
            },
            None => {
                parent.insert_before(&child_item, None).unwrap();
            }
        }
        self.copy_parent_from_rel(&child, &ref_id);
    }

    fn add_child(&mut self, parent: RealDomId, child: RealDomId) {
        let parent_item = self.get_node(&parent).unwrap();
        let child_item = self.get_node(&child).unwrap();

        parent_item.append_child(&child_item).unwrap();
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
        }
    }

    fn insert_css(&self, selector: &str, value: &str) {
        let style = self.document.create_element("style").unwrap();
        let content = self.document.create_text_node(format!("{} {{ {} }}", selector, value).as_str());
        style.append_child(&content).unwrap();

        self.head.append_child(&style).unwrap();
    }
}


pub struct DomDriverBrowser {
    driver: Rc<BoxRefCell<DomDriverBrowserInner>>,
}

impl DomDriverBrowser {
    pub fn new() -> DomDriver {
        let driver = DomDriverBrowserInner::new();

        let dom_driver_browser = DomDriverBrowser {
            driver,
        };

        let driver = DomDriver::new(dom_driver_browser, Box::new(|fut: Pin<Box<dyn Future<Output = ()> + 'static>>| {
            wasm_bindgen_futures::spawn_local(fut);
        }));

        driver
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

    fn insert_as_first_child(&self, parent: RealDomId, child: RealDomId) {
        self.driver.change((parent, child), |state, (parent, child)| {
            state.insert_as_first_child(parent, child);
        });
    }

    fn insert_before(&self, ref_id: RealDomId, child: RealDomId) {
        self.driver.change((ref_id, child), |state, (ref_id, child)| {
            state.insert_before(ref_id, child);
        });
    }

    fn insert_after(&self, ref_id: RealDomId, child: RealDomId) {
        self.driver.change((ref_id, child), |state, (ref_id, child)| {
            state.insert_after(ref_id, child);
        });
    }

    fn add_child(&self, parent: RealDomId, child: RealDomId) {
        self.driver.change((parent, child), |state, (parent, child)| {
            state.add_child(parent, child);
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

    fn fetch(&self, method: FetchMethod, url: String, headers: Option<HashMap<String, String>>, body: Option<String>) -> Pin<Box<dyn Future<Output=Result<String, FetchError>> + 'static>> {
        fetch::fetch(method, url, headers, body)
    }
}
