#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use web_sys::{Document, Element, Text, HtmlHeadElement, Node};
use std::rc::Rc;
use std::collections::HashMap;

use virtualdom::computed::BoxRefCell::BoxRefCell;
use virtualdom::vdom::driver::DomDriver::DomDriverTrait;
use virtualdom::vdom::models::RealDomId::RealDomId;

use wasm_bindgen::JsCast;
use dom_event::{DomEventDisconnect, DomEventMouse};

mod dom_event;

fn get_document() -> (Document, HtmlHeadElement) {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let head = document.head().unwrap();
    (document, head)
}

fn create_root(document: &Document) -> Element {
    let body = document.body().expect("document should have a body");
    let root = document.create_element("div").unwrap();
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

pub struct DomDriverBrowserInner {
    document: Document,
    head: HtmlHeadElement,
    elements: HashMap<RealDomId, ElementItem>,
    eventsOnClick: HashMap<RealDomId, DomEventDisconnect>,
}

impl Default for DomDriverBrowserInner {
    fn default() -> Self {
        let (document, head) = get_document();
        let root = create_root(&document);

        let mut elements = HashMap::new();    
        elements.insert(RealDomId::root(), ElementItem::Element { node: root });

        Self {
            document,
            head,
            elements,
            eventsOnClick: HashMap::new(),
        }
    }
}

impl DomDriverBrowserInner {
    fn createNode(&mut self, id: RealDomId, name: &'static str) {
        let node = self.document.create_element(name).unwrap();
        let id_str = format!("{}", id.to_u64());
        node.set_attribute("debug-id", id_str.as_str()).unwrap();
        self.elements.insert(id, ElementItem::Element { node });
    }

    fn createText(&mut self, id: RealDomId, value: &str) {
        let text = self.document.create_text_node(value);
        self.elements.insert(id, ElementItem::Text { text });
    }

    fn setAttr(&mut self, id: RealDomId, name: &'static str, value: &str) {
        let elem = self.elements.get_mut(&id);

        if let Some(elem) = elem {
            match elem {
                ElementItem::Element { node } => {
                    node.set_attribute(name, value).unwrap();
                },
                ElementItem::Text { .. } => {
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
                ElementItem::Element { node } => {
                    node.remove_attribute(name).unwrap();
                },
                ElementItem::Text { .. } => {
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
                ElementItem::Element { node } => {
                    node.remove();
                },
                ElementItem::Text { text } => {
                    text.remove();
                }
            }
            return;
        }

        log::error!("Missing element with id={}", id);
    }

    fn get_node(&self, refId: RealDomId) -> Option<Node> {
        let child_item = self.elements.get(&refId);

        match child_item {
            Some(ElementItem::Element { node }) => {
                let node = node.clone().dyn_into::<Node>().unwrap();
                Some(node)
            },
            Some(ElementItem::Text { text }) => {
                let node = text.clone().dyn_into::<Node>().unwrap();
                Some(node)
            },
            None => {
                log::error!("no element was found id={}", refId);
                None
            }
        }
    }

    fn insertAsFirstChild(&self, parent: RealDomId, child: RealDomId) {
        let parent_item = self.get_node(parent).unwrap();
        let child_item = self.get_node(child).unwrap();

        parent_item.insert_before(&child_item, None).unwrap();
    }

    fn insertBefore(&self, refId: RealDomId, child: RealDomId) {
        let refId_item = self.get_node(refId).unwrap();
        let child_item = self.get_node(child).unwrap();

        let parent: Node = refId_item.parent_node().unwrap();

        parent.insert_before(&child_item, Some(&refId_item)).unwrap();
    }

    fn insertAfter(&self, refId: RealDomId, child: RealDomId) {
        let refId_item = self.get_node(refId).unwrap();
        let child_item = self.get_node(child).unwrap();

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
    }

    fn addChild(&mut self, parent: RealDomId, child: RealDomId) {
        let parent_item = self.get_node(parent).unwrap();
        let child_item = self.get_node(child).unwrap();

        parent_item.append_child(&child_item).unwrap();
    }

    fn setOnClick(&mut self, node_id: RealDomId, onClick: Option<Rc<dyn Fn()>>) {
        
        match onClick {
            Some(onClick) => {
                let node_item = self.get_node(node_id.clone()).unwrap();
                let clouser = DomEventMouse::new(move |_event: &web_sys::MouseEvent| {
                    onClick();
                });

                let disconnect = clouser.append_to_mousedown(&node_item);
                
                self.eventsOnClick.insert(node_id, disconnect);
            },
            None => {
                self.eventsOnClick.remove(&node_id);
            }
        }
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

impl Default for DomDriverBrowser {
    fn default() -> Self {
        let driver = Rc::new(
            BoxRefCell::new(
                DomDriverBrowserInner::default()
            )
        );

        Self {
            driver,
        }
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
}
