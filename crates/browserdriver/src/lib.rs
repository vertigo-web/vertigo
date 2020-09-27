#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use wasm_bindgen::prelude::*;
use std::rc::Rc;
use std::collections::HashMap;

use virtualdom::computed::BoxRefCell::BoxRefCell;
use virtualdom::vdom::DomDriver::DomDriver::DomDriverTrait;
use virtualdom::vdom::models::RealDomId::RealDomId;

use wasm_bindgen::JsValue;
use crate::event::EventModel;

mod event;

#[wasm_bindgen(module = "/src/driver.js")]
extern "C" {
    fn consoleLog(message: &str);
    fn startDriverLoop(closure: &Closure<dyn FnMut()>);

    fn createNode(id: u64, name: &str);
    fn createText(id: u64, value: &str);
    fn createComment(id: u64, value: &str);
    fn setAttr(id: u64, key: &str, value: &str);
    fn removeAttr(id: u64, name: &str);
    fn remove(id: u64);
    fn insertAsFirstChild(parent: u64, child: u64);

    fn insertBefore(refId: u64, child: u64);
    fn insertAfter(refId: u64, child: u64);
    fn addChild(parent: u64, child: u64);

    fn getEventData() -> JsValue;

    fn insertCss(class: String, value: String);
}

struct DriverJS {}

impl DriverJS {
    unsafe fn consoleLog(message: &str) {
        consoleLog(message);
    }

    unsafe fn startDriverLoop(closure: &Closure<dyn FnMut()>) {
        startDriverLoop(closure);
    }

    unsafe fn createNode(id: u64, name: &str) {
        createNode(id, name);
    }

    unsafe fn createText(id: u64, value: &str) {
        createText(id, value);
    }

    unsafe fn createComment(id: u64, value: &str) {
        createComment(id, value);
    }

    unsafe fn setAttr(id: u64, key: &str, value: &str) {
        setAttr(id, key, value);
    }

    unsafe fn removeAttr(id: u64, name: &str) {
        removeAttr(id, name)
    }

    unsafe fn remove(id: u64) {
        remove(id);
    }

    unsafe fn insertAsFirstChild(parent: u64, child: u64) {
        insertAsFirstChild(parent, child);
    }

    unsafe fn insertBefore(refId: u64, child: u64) {
        insertBefore(refId, child);
    }

    unsafe fn insertAfter(refId: u64, child: u64) {
        insertAfter(refId, child);
    }

    unsafe fn addChild(parent: u64, child: u64) {
        addChild(parent, child);
    }

    unsafe fn getEventData() -> JsValue {
        getEventData()
    }

    unsafe fn insertCss(class: String, value: String) {
        insertCss(class, value)
    }
}

pub struct DomDriverBrowserInner {
    parent: BoxRefCell<HashMap<u64, u64>>,                                  //child -> parent
    dataOnClick: BoxRefCell<HashMap<u64, Rc<dyn Fn()>>>,
}

impl DomDriverBrowserInner {
    pub fn new() -> DomDriverBrowserInner {
        DomDriverBrowserInner {
            parent: BoxRefCell::new(HashMap::new()),
            dataOnClick: BoxRefCell::new(HashMap::new()),
        }
    }
}

impl DomDriverBrowserInner {
    fn createNode(&self, id: RealDomId, name: &'static str) {
        unsafe {
            DriverJS::createNode(id.to_u64(), name);
        }
    }

    fn createText(&self, id: RealDomId, value: &String) {
        unsafe {
            DriverJS::createText(id.to_u64(), value.as_str());
        }
    }

    fn createComment(&self, id: RealDomId, value: &String) {
        unsafe {
            DriverJS::createComment(id.to_u64(), value.as_str());
        }
    }

    fn setAttr(&self, id: RealDomId, key: &'static str, value: &String) {
        unsafe {
            DriverJS::setAttr(id.to_u64(), key, value.as_str());
        }
    }

    fn removeAttr(&self, id: RealDomId, name: &'static str) {
        unsafe {
            DriverJS::removeAttr(id.to_u64(), name);
        }
    }

    fn remove(&self, id: RealDomId) {
        let id = id.to_u64();
        unsafe {
            DriverJS::remove(id);
        }

        self.dataOnClick.change(&id, |state, id| {
            state.remove(id);
        });

        self.parent.change(&id, |state, id| {
            state.remove(&id);
        })
    }

    fn setParent(&self, parent: RealDomId, child: RealDomId) {
        self.parent.change((parent, child), |state, (parent, child)| {
            state.insert(child.to_u64(), parent.to_u64());
        })
    }

    fn setRel(&self, relId: RealDomId, child: RealDomId) {
        self.parent.change((relId, child), |state, (relId, child)| {
            let relId = relId.to_u64();
            let child = child.to_u64();

            let parent = state.get(&relId);

            let parent = *(parent.unwrap());                       //TODO - koniecznie musi być ten idk
            state.insert(child, parent);
        });
    }

    fn insertAsFirstChild(&self, parent: RealDomId, child: RealDomId) {
        unsafe {
            DriverJS::insertAsFirstChild(parent.to_u64(), child.to_u64());
        }
        self.setParent(parent, child);
    }

    fn insertBefore(&self, refId: RealDomId, child: RealDomId) {
        unsafe {
            DriverJS::insertBefore(refId.to_u64(), child.to_u64());
        }
        self.setRel(refId, child);
    }

    fn insertAfter(&self, refId: RealDomId, child: RealDomId) {
        unsafe {
            DriverJS::insertAfter(refId.to_u64(), child.to_u64());
        }
        self.setRel(refId, child);
    }

    fn addChild(&self, parent: RealDomId, child: RealDomId) {
        unsafe {
            DriverJS::addChild(parent.to_u64(), child.to_u64());
        }
        self.setParent(parent, child);
    }

    fn setOnClick(&self, node: RealDomId, onClick: Option<Rc<dyn Fn()>>) {
        self.dataOnClick.change((node, onClick), |state, (node, onClick)| {
            let id = node.to_u64();

            match onClick {
                Some(onClick) => {
                    state.insert(id, onClick);
                },
                None => {
                    state.remove(&id);
                }
            };
        });
    }

    fn getEvent(&self, nodeId: &u64) -> Option<Rc<dyn Fn()>> {
        self.dataOnClick.getWithContext(nodeId, |state, nodeId| {
            state.get(nodeId).map(|item| item.clone())
        })
    }

    fn getParentNode(&self, childId: &u64) -> Option<u64> {
        self.parent.getWithContext(childId, |state, childId| {
            let parent = state.get(&childId);
            parent.map(|item| *item)
        })
    }

    fn insertCss(&self, class: String, value: String) {
        unsafe {
            DriverJS::insertCss(class, value)
        }
    }

    fn sendEvent(&self, event: &EventModel) {
        log::info!("Przyszedł event {:?}", event);

        match event {
            EventModel::OnClick { nodeId} => {
                let mut nodeId = *nodeId;

                while nodeId != 1 {
                    let event = self.getEvent(&nodeId);

                    if let Some(event) = event {
                        event();
                        return;
                    }

                    let parent = self.getParentNode(&nodeId);

                    if let Some(parent) = parent {
                        if parent == 1 {
                            log::info!("sendEvent - trafiono na root");
                            return;
                        }

                        nodeId = parent;

                    } else {
                        log::info!("sendEvent - nie znaleziono roota");
                    }
                }
            }
        }
    }

    fn fromCallback(&self) {
        let data = unsafe {
            DriverJS::getEventData()
        };

        let result: Result<Vec<EventModel>, serde_json::error::Error> = data.into_serde::<Vec<EventModel>>();

        match result {
            Ok(event) => {

                //TODO - tranzakcja start

                for item in event.iter() {
                    let item: &EventModel = item;
                    self.sendEvent(item);
                }

                //złapać tranzakcją
                    //w tej tranzakcji, w petli aktualizowac

                //TODO - tranzakcja stop
            },
            Err(err) => {
                log::error!("Przyszedł zepsuty event {:?}", err);
            }
        };
    }
}

pub struct DomDriverBrowserRc {
    inner: Rc<DomDriverBrowserInner>,
}

impl DomDriverBrowserRc {
    fn fromCallback(&self) {
        self.inner.fromCallback();
    }

    fn new() -> DomDriverBrowserRc {
        DomDriverBrowserRc {
            inner: Rc::new(DomDriverBrowserInner::new())
        }
    }
}

impl Clone for DomDriverBrowserRc {
    fn clone(&self) -> Self {
        DomDriverBrowserRc {
            inner: self.inner.clone()
        }
    }
}


pub struct DomDriverBrowser {
    driver: DomDriverBrowserRc,
    _callFromJS: Rc<Closure<dyn FnMut()>>,
}

impl DomDriverBrowser {
    pub fn new() -> DomDriverBrowser {

        let driver = DomDriverBrowserRc::new();

        let callFromJS: Closure<dyn FnMut()> = {
            let driver = driver.clone();
            let back = Closure::new(move || {
                driver.fromCallback();
            });

            unsafe {
                DriverJS::startDriverLoop(&back);
            }

            back
        };

        DomDriverBrowser {
            driver,
            _callFromJS: Rc::new(callFromJS),
        }
    }

    pub fn consoleLog(&self, message: &str) {
        unsafe {
            DriverJS::consoleLog(message);
        }
    }
}

impl DomDriverTrait for DomDriverBrowser {
    fn createNode(&self, id: RealDomId, name: &'static str) {
        self.driver.inner.createNode(id, name);
    }

    fn createText(&self, id: RealDomId, value: &String) {
        self.driver.inner.createText(id, value);
    }

    fn createComment(&self, id: RealDomId, value: &String) {
        self.driver.inner.createComment(id, value);
    }

    fn setAttr(&self, id: RealDomId, key: &'static str, value: &String) {
        self.driver.inner.setAttr(id, key, value);
    }

    fn removeAttr(&self, id: RealDomId, name: &'static str) {
        self.driver.inner.removeAttr(id, name);
    }

    fn remove(&self, id: RealDomId) {
        self.driver.inner.remove(id);
    }

    fn insertAsFirstChild(&self, parent: RealDomId, child: RealDomId) {
        self.driver.inner.insertAsFirstChild(parent, child);
    }

    fn insertBefore(&self, refId: RealDomId, child: RealDomId) {
        self.driver.inner.insertBefore(refId, child);
    }

    fn insertAfter(&self, refId: RealDomId, child: RealDomId) {
        self.driver.inner.insertAfter(refId, child);
    }

    fn addChild(&self, parent: RealDomId, child: RealDomId) {
        self.driver.inner.addChild(parent, child);
    }

    fn insertCss(&self, class: String, value: String) {
        self.driver.inner.insertCss(class, value);
    }

    fn setOnClick(&self, node: RealDomId, onClick: Option<Rc<dyn Fn()>>) {
        self.driver.inner.setOnClick(node, onClick);
    }
}
