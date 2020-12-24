use std::rc::Rc;
use crate::virtualdom::models::{
    RealDomId::RealDomId,
};
use std::pin::Pin;
use std::future::Future;
use std::collections::HashMap;

#[derive(Debug)]
pub enum FetchMethod {
    GET,
    POST,
}

impl FetchMethod {
    pub fn to_string(&self) -> &str {
        match self {
            FetchMethod::GET => "GET",
            FetchMethod::POST => "POST",
        }
    }
}

pub enum FetchError {
    Error,
}

pub enum EventCallback {
    OnClick {
        callback: Option<Rc<dyn Fn()>>,
    },
    OnInput {
        callback: Option<Rc<dyn Fn(String)>>,
    },
                //mouseenter
    OnMouseEnter {
        callback: Option<Rc<dyn Fn()>>,
    },
                //mouseleave
    OnMouseLeave {
        callback: Option<Rc<dyn Fn()>>,
    },
}

impl EventCallback {
    pub fn to_string(&self) -> &str {
        match self {
            EventCallback::OnClick { callback} => {
                if callback.is_some() {
                    "onClick set"
                } else {
                    "onClick clear"
                }
            },
            EventCallback::OnInput { callback } =>{
                if callback.is_some() {
                    "onInput set"
                } else {
                    "onInput clear"
                }
            },
            EventCallback::OnMouseEnter { callback } =>{
                if callback.is_some() {
                    "onMouseEnter set"
                } else {
                    "onMouseEnter clear"
                }
            },
            EventCallback::OnMouseLeave { callback } =>{
                if callback.is_some() {
                    "onMouseLeave set"
                } else {
                    "onMouseLeave clear"
                }
            },
        }
    }
}

const SHOW_LOG: bool = false;

pub trait DomDriverTrait {
    fn create_node(&self, id: RealDomId, name: &'static str);
    fn create_text(&self, id: RealDomId, value: &str);
    fn update_text(&self, id: RealDomId, value: &str);
    fn set_attr(&self, id: RealDomId, key: &'static str, value: &str);
    fn remove_attr(&self, id: RealDomId, name: &'static str);
    fn remove(&self, id: RealDomId);
    fn insert_as_first_child(&self, parent: RealDomId, child: RealDomId);
    fn insert_before(&self, ref_id: RealDomId, child: RealDomId);
    fn insert_after(&self, ref_id: RealDomId, child: RealDomId);
    fn add_child(&self, parent: RealDomId, child: RealDomId);
    fn insert_css(&self, selector: String, value: String);
    fn set_event(&self, node: RealDomId, callback: EventCallback);
    fn fetch(&self, method: FetchMethod, url: String, headers: Option<HashMap<String, String>>, body: Option<String>) -> Pin<Box<dyn Future<Output=Result<String, FetchError>> + 'static>>; 
}

type Executor = Box<dyn Fn(Pin<Box<dyn Future<Output = ()> + 'static>>) -> ()>;

pub struct DomDriver {
    driver: Rc<dyn DomDriverTrait>,
    spawn_local_executor: Rc<Executor>,
}

impl DomDriver {
    pub fn new<
        T: DomDriverTrait + 'static,
    >(driver: T, spawn_local: Executor) -> DomDriver {
        DomDriver {
            driver: Rc::new(driver),
            spawn_local_executor: Rc::new(spawn_local)
        }
    }
}

impl Clone for DomDriver {
    fn clone(&self) -> DomDriver {
        DomDriver {
            driver: self.driver.clone(),
            spawn_local_executor: self.spawn_local_executor.clone(),
        }
    }
}

fn show_log(message: String) {
    if SHOW_LOG {
        log::info!("{}", message);
    }
}

impl DomDriver {
    pub fn spawn_local<F>(&self, future: F)
        where F: Future<Output = ()> + 'static {
        
            let fur = Box::pin(future);

            let spawn_local_executor = self.spawn_local_executor.clone();
            spawn_local_executor(fur)
}
    pub fn create_node(&self, id: RealDomId, name: &'static str) {
        show_log(format!("create_node {} {}", id, name));
        self.driver.create_node(id, name);
    }

    pub fn create_text(&self, id: RealDomId, value: &str) {
        show_log(format!("create_text {} {}", id, value));
        self.driver.create_text(id, value);
    }

    pub fn update_text(&self, id: RealDomId, value: &str) {
        show_log(format!("update_text {} {}", id, value));
        self.driver.update_text(id, value);
    }

    pub fn set_attr(&self, id: RealDomId, key: &'static str, value: &str) {
        show_log(format!("set_attr {} {} {}", id, key, value));
        self.driver.set_attr(id, key, value);
    }

    pub fn remove_attr(&self, id: RealDomId, name: &'static str) {
        show_log(format!("remove_attr {} {}", id, name));
        self.driver.remove_attr(id, name);
    }

    pub fn remove(&self, id: RealDomId) {
        show_log(format!("remove {}", id));
        self.driver.remove(id);
    }

    pub fn insert_as_first_child(&self, parent: RealDomId, child: RealDomId) {
        show_log(format!("insert_as_first_child parent={} child={}", parent, child));
        self.driver.insert_as_first_child(parent, child);
    }

    pub fn insert_before(&self, ref_id: RealDomId, child: RealDomId) {
        show_log(format!("insert_before refId={} child={}", ref_id, child));
        self.driver.insert_before(ref_id, child);
    }

    pub fn insert_after(&self, ref_id: RealDomId, child: RealDomId) {
        show_log(format!("insert_after refId={} child={}", ref_id, child));
        self.driver.insert_after(ref_id, child);
    }

    pub fn add_child(&self, parent: RealDomId, child: RealDomId) {
        show_log(format!("add_child parent={} child={}", parent, child));
        self.driver.add_child(parent, child);
    }

    pub fn insert_css(&self, selector: String, value: String) {
        show_log(format!("insert_css selector={} value={}", selector, value));
        self.driver.insert_css(selector, value);
    }

    pub fn set_event(&self, node: RealDomId, callback: EventCallback) {
        show_log(format!("set_event {} {}", node, callback.to_string()));
        self.driver.set_event(node, callback);
    }

    pub fn fetch(&self, method: FetchMethod, url: String, headers: Option<HashMap<String, String>>, body: Option<String>) -> Pin<Box<dyn Future<Output=Result<String, FetchError>> + 'static>> {
        show_log(format!("fetch {:?} {}", method, url));
        self.driver.fetch(method, url, headers, body)
    }
}
