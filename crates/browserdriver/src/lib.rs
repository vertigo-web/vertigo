#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use web_sys::{Document, Element, Text, HtmlHeadElement, Node};
use std::rc::Rc;
use std::collections::HashMap;

use virtualdom::{DomDriver, computed::BoxRefCell};
use virtualdom::{DomDriverTrait, FetchMethod, FetchError};
use virtualdom::RealDomId;

use wasm_bindgen::JsCast;
use dom_event::{DomEventDisconnect, DomEventMouse};

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
    pub fn fromNode(node: Element) -> ElementItem {
        ElementItem::Element { node }
    }

    pub fn fromText(text: Text) -> ElementItem {
        ElementItem::Text { text }
    }
}

struct ElementWrapper {
    item: ElementItem,
    onClick: Option<Rc<dyn Fn()>>,
}

impl ElementWrapper {
    pub fn fromNode(node: Element) -> ElementWrapper {
        ElementWrapper {
            item: ElementItem::fromNode(node),
            onClick: None,
        }
    }

    pub fn fromText(text: Text) -> ElementWrapper {
        ElementWrapper {
            item: ElementItem::fromText(text),
            onClick: None,
        }
    }
}

fn find_event(inner: &Rc<BoxRefCell<DomDriverBrowserInner>>, id: u64) -> Option<Rc<dyn Fn()>> {
    let id = RealDomId::from_u64(id);

    let on_click = inner.getWithContext(
        id,
        |state, id| -> Option<Rc<dyn Fn()>> {
            let mut wsk = id;
            let mut count = 0;

            loop {
                count += 1;

                if count > 100 {
                    log::error!("Too many nested levels");
                    return None;
                }

                let item = state.elements.get(&wsk).unwrap();

                if let Some(on_click) = &item.onClick {
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

pub struct DomDriverBrowserInner {
    document: Document,
    head: HtmlHeadElement,
    elements: HashMap<RealDomId, ElementWrapper>,
    child_parent: HashMap<RealDomId, RealDomId>,            //child -> parent
    _mouse_down: Option<DomEventDisconnect>,
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
                    _mouse_down: None
                }
            )
        );

        let clouser = {
            let inner = inner.clone();

            DomEventMouse::new(move |event: &web_sys::MouseEvent| {
                // log::info!("event click ... {:?}", event);

                let target = event.target().unwrap();
                let element = target.dyn_ref::<Element>().unwrap();

                let option_id: Option<String> = (*element).get_attribute("data-id");
                let id: u64 = option_id.unwrap().parse::<u64>().unwrap();

                let event_to_run = find_event(&inner, id);

                if let Some(event_to_run) = event_to_run {
                    event_to_run();
                }
            })
        };

        let mouse_down = clouser.append_to_mousedown(&root);

        inner.change(
            (mouse_down, root_id, root),
            |state, (mouse_down, root_id, root)| {
                state.elements.insert(root_id, ElementWrapper::fromNode(root));
                state._mouse_down = Some(mouse_down);
            }
        );

        inner
    }

    fn createNode(&mut self, id: RealDomId, name: &'static str) {
        let node = create_node(&self.document, &id, name);
        self.elements.insert(id, ElementWrapper::fromNode(node));
    }

    fn createText(&mut self, id: RealDomId, value: &str) {
        let text = self.document.create_text_node(value);
        self.elements.insert(id, ElementWrapper::fromText(text));
    }

    fn setAttr(&mut self, id: RealDomId, name: &'static str, value: &str) {
        let elem = self.elements.get_mut(&id);

        if let Some(elem) = elem {
            match elem {
                ElementWrapper { item: ElementItem::Element { node }, ..} => {
                    node.set_attribute(name, value).unwrap();
                },
                ElementWrapper { item: ElementItem::Text { .. }, ..} => {
                    log::error!("Cannot set attribute on a text node id={}", id);
                }
            }
            return;
        }

        log::error!("Missing element with id={}", id);
    }

    fn removeAttr(&mut self, id: RealDomId, name: &'static str) {
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

    fn get_node(&self, refId: &RealDomId) -> Option<Node> {
        let child_item = self.elements.get(refId);

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
                log::error!("no element was found id={}", refId);
                None
            }
        }
    }

    fn copy_parent_from_rel(&mut self, child: &RealDomId, rel: &RealDomId) {
        let rel_parent = self.child_parent.get(rel).unwrap().clone();
        self.child_parent.insert(child.clone(), rel_parent);
    }

    fn insertAsFirstChild(&mut self, parent: RealDomId, child: RealDomId) {
        let parent_item = self.get_node(&parent).unwrap();
        let child_item = self.get_node(&child).unwrap();

        parent_item.insert_before(&child_item, None).unwrap();
        self.child_parent.insert(child, parent);
    }

    fn insertBefore(&mut self, refId: RealDomId, child: RealDomId) {
        let refId_item = self.get_node(&refId).unwrap();
        let child_item = self.get_node(&child).unwrap();

        let parent: Node = refId_item.parent_node().unwrap();

        parent.insert_before(&child_item, Some(&refId_item)).unwrap();
        self.copy_parent_from_rel(&child, &refId);
    }

    fn insertAfter(&mut self, refId: RealDomId, child: RealDomId) {
        let refId_item = self.get_node(&refId).unwrap();
        let child_item = self.get_node(&child).unwrap();

        let parent: Node = refId_item.parent_node().unwrap();
        let next: Option<Node> = refId_item.next_sibling();

        match next {
            Some(next) => {
                parent.insert_before(&child_item, Some(&next)).unwrap();
            },
            None => {
                parent.insert_before(&child_item, None).unwrap();
            }
        }
        self.copy_parent_from_rel(&child, &refId);
    }

    fn addChild(&mut self, parent: RealDomId, child: RealDomId) {
        let parent_item = self.get_node(&parent).unwrap();
        let child_item = self.get_node(&child).unwrap();

        parent_item.append_child(&child_item).unwrap();
        self.child_parent.insert(child, parent);
    }

    fn setOnClick(&mut self, node_id: RealDomId, onClick: Option<Rc<dyn Fn()>>) {
        let item = self.elements.get_mut(&node_id).unwrap();
        item.onClick = onClick;
    }

    fn insertCss(&self, selector: String, value: String) {
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

        let domDriverBrowser = DomDriverBrowser {
            driver,
        };

        let driver = DomDriver::new(domDriverBrowser, Box::new(|fut: Pin<Box<dyn Future<Output = ()> + 'static>>| {
            wasm_bindgen_futures::spawn_local(fut);
        }));

        driver
    }
}

impl DomDriverTrait for DomDriverBrowser {
    fn createNode(&self, id: RealDomId, name: &'static str) {
        self.driver.change((id, name), |state, (id, name)| {
            state.createNode(id, name);
        });
    }

    fn createText(&self, id: RealDomId, value: &str) {
        self.driver.change((id, value), |state, (id, value)| {
            state.createText(id, value);
        });
    }

    fn setAttr(&self, id: RealDomId, key: &'static str, value: &str) {
        self.driver.change((id, key, value), |state, (id, key, value)| {
            state.setAttr(id, key, value);
        });
    }

    fn removeAttr(&self, id: RealDomId, name: &'static str) {
        self.driver.change((id, name), |state, (id, name)| {
            state.removeAttr(id, name);
        });
    }

    fn remove(&self, id: RealDomId) {
        self.driver.change(id, |state, id| {
            state.remove(id);
        });
    }

    fn insertAsFirstChild(&self, parent: RealDomId, child: RealDomId) {
        self.driver.change((parent, child), |state, (parent, child)| {
            state.insertAsFirstChild(parent, child);
        });
    }

    fn insertBefore(&self, refId: RealDomId, child: RealDomId) {
        self.driver.change((refId, child), |state, (refId, child)| {
            state.insertBefore(refId, child);
        });
    }

    fn insertAfter(&self, refId: RealDomId, child: RealDomId) {
        self.driver.change((refId, child), |state, (refId, child)| {
            state.insertAfter(refId, child);
        });
    }

    fn addChild(&self, parent: RealDomId, child: RealDomId) {
        self.driver.change((parent, child), |state, (parent, child)| {
            state.addChild(parent, child);
        });
    }

    fn insertCss(&self, class: String, value: String) {
        self.driver.change((class, value), |state, (class, value)| {
            state.insertCss(class, value);
        });
    }

    fn setOnClick(&self, node: RealDomId, onClick: Option<Rc<dyn Fn()>>) {
        self.driver.change((node, onClick), |state, (node, onClick)| {
            state.setOnClick(node, onClick);
        });
    }

    fn fetch(&self, method: FetchMethod, url: String, headers: Option<HashMap<String, String>>, body: Option<String>) -> Pin<Box<dyn Future<Output=Result<String, FetchError>> + 'static>> {
        fetch::fetch(method, url, headers, body)
    }
}
