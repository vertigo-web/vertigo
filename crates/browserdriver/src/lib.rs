use wasm_bindgen::prelude::*;
use std::rc::Rc;

use virtualdom::vdom::DomDriver::DomDriver::DomDriverTrait;
use virtualdom::vdom::models::RealDomId::RealDomId;

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
    fn removeAllChild(id: u64);
    fn insertAsFirstChild(parent: u64, child: u64);

    fn insertBefore(refId: u64, child: u64);
    fn insertAfter(refId: u64, child: u64);
    fn addChild(parent: u64, child: u64);
    
}

pub struct DomDriverBrowserInner {

}

impl DomDriverBrowserInner {
    fn createNode(&self, id: RealDomId, name: &String) {
        createNode(id.to_u64(), name.as_str());
    }

    fn createText(&self, id: RealDomId, value: &String) {
        createText(id.to_u64(), value.as_str());
    }

    fn createComment(&self, id: RealDomId, value: &String) {
        createComment(id.to_u64(), value.as_str());
    }

    fn setAttr(&self, id: RealDomId, key: &String, value: &String) {
        setAttr(id.to_u64(), key.as_str(), value.as_str());
    }

    fn removeAttr(&self, id: RealDomId, name: &String) {
        removeAttr(id.to_u64(), name.as_str());
    }

    fn remove(&self, id: RealDomId) {
        remove(id.to_u64());
    }

    fn removeAllChild(&self, id: RealDomId) {
        removeAllChild(id.to_u64());
    }

    fn insertAsFirstChild(&self, parent: RealDomId, child: RealDomId) {
        insertAsFirstChild(parent.to_u64(), child.to_u64());
    }

    fn insertBefore(&self, refId: RealDomId, child: RealDomId) {
        insertBefore(refId.to_u64(), child.to_u64());
    }

    fn insertAfter(&self, refId: RealDomId, child: RealDomId) {
        insertAfter(refId.to_u64(), child.to_u64());
    }

    fn addChild(&self, parent: RealDomId, child: RealDomId) {
        addChild(parent.to_u64(), child.to_u64());
    }
}

pub struct DomDriverBrowserRc {
    inner: Rc<DomDriverBrowserInner>,
}

impl DomDriverBrowserRc {
    fn fromCallback(&self) {
        log::info!("callback z drivera");
    }

    fn new() -> DomDriverBrowserRc {
        DomDriverBrowserRc {
            inner: Rc::new(DomDriverBrowserInner {

            })
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

impl DomDriverTrait for DomDriverBrowserRc {
    fn createNode(&self, id: RealDomId, name: &String) {
        self.inner.createNode(id, name);
    }

    fn createText(&self, id: RealDomId, value: &String) {
        self.inner.createText(id, value);
    }

    fn createComment(&self, id: RealDomId, value: &String) {
        self.inner.createComment(id, value);
    }

    fn setAttr(&self, id: RealDomId, key: &String, value: &String) {
        self.inner.setAttr(id, key, value);
    }

    fn removeAttr(&self, id: RealDomId, name: &String) {
        self.inner.removeAttr(id, name);
    }

    fn remove(&self, id: RealDomId) {
        self.inner.remove(id);
    }

    fn removeAllChild(&self, id: RealDomId) {
        self.inner.removeAllChild(id);
    }

    fn insertAsFirstChild(&self, parent: RealDomId, child: RealDomId) {
        self.inner.insertAsFirstChild(parent, child);
    }

    fn insertBefore(&self, refId: RealDomId, child: RealDomId) {
        self.inner.insertBefore(refId, child);
    }

    fn insertAfter(&self, refId: RealDomId, child: RealDomId) {
        self.inner.insertAfter(refId, child);
    }

    fn addChild(&self, parent: RealDomId, child: RealDomId) {
        self.inner.addChild(parent, child);
    }
}


pub struct DomDriverBrowser {
    pub driver: DomDriverBrowserRc,
    _callFromJS: Closure<dyn FnMut()>,
}

impl DomDriverBrowser {
    pub fn new() -> DomDriverBrowser {

        let driver = DomDriverBrowserRc::new();

        let callFromJS: Closure<dyn FnMut()> = {
            let driver = driver.clone();
            let back = Closure::new(move || {
                driver.fromCallback();
            });
    
            startDriverLoop(&back);
    
            back
        };

        DomDriverBrowser {
            driver,
            _callFromJS: callFromJS,
        }
    }

    pub fn consoleLog(&self, message: &str) {
        consoleLog(message);
    }
}

//Moze te bindowania i ten DomDriverBrowser da sie zamknac w jednym module w bibliotece

/*
    jak przyjdzie jakis zdarzenie od uzytkownika
    zapisujemy je w jakiejs tablicy po stronie jsowego drivera
    wysyłamy sygnał zwrotny do modulu rustowego ze czekaja nowe zdarzenia
    modul rustowy pobiera sobie te nowe informaje na temat eventu z jsa i zmienia odpowiedni stan
*/
